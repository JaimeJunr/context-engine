#!/usr/bin/env ruby
# frozen_string_literal: true

# read-xml5-anbima.rb - Script da skill read-xml5-anbima: leitura e análise de arquivos XML5 ANBIMA
# Utiliza Nokogiri para parsing e validação conforme padrão XML5 da ANBIMA (alinhado ao fluxo Performit)

require 'nokogiri'
require 'json'
require 'optparse'
require 'fileutils'

# Namespaces XML5 ANBIMA (alinhado ao fluxo Performit: Grails/Rails/frontend)
XML5_NAMESPACE = 'urn:iso:std:iso:20022:tech:xsd:semt.003.001.04'
HEAD_NAMESPACE = 'urn:iso:std:iso:20022:tech:xsd:head.001.001.01'

# Classe principal para análise de XML5 ANBIMA
class XML5Analyzer
  def initialize(file_path, validate: true, extract: nil, format: 'summary')
    @file_path = file_path
    @validate = validate
    @extract_sections = extract&.split(',')&.map(&:strip)
    @format = format
    @namespace = XML5_NAMESPACE
    @head_namespace = HEAD_NAMESPACE
    @errors = []
    @warnings = []
  end

  def analyze
    validate_file_exists
    doc = load_and_parse_xml
    return nil unless doc

    # Nó de trabalho: mensagem semt (igual ao que o sistema usa: Document > SctiesBalAcctgRpt)
    @message_root = resolve_message_root(doc)

    analysis = {
      bah: extract_bah(doc),
      paginacao: extract_pagination(doc),
      prestadores: extract_providers(doc),
      detalhes_gerais: extract_general_details(doc),
      carteira: extract_portfolio_details(doc),
      ativos: extract_assets(doc),
      despesas: extract_expenses(doc),
      validacao: {
        estrutura: validate_structure(doc),
        estrutura_erros: @structure_errors || [],
        campos_obrigatorios: validate_required_fields(doc),
        calculos: validate_calculations(doc)
      }
    }

    # Filtrar seções se --extract foi especificado
    if @extract_sections
      analysis = filter_sections(analysis, @extract_sections)
    end

    format_output(analysis)
  end

  private

  def validate_file_exists
    unless File.exist?(@file_path)
      raise "Arquivo não encontrado: #{@file_path}"
    end

    unless File.readable?(@file_path)
      raise "Arquivo não pode ser lido: #{@file_path}"
    end
  end

  def load_and_parse_xml
    xml_content = File.read(@file_path, encoding: 'UTF-8')
    # Alinhado ao Anbima5ImportService (Grails): remover BOM para evitar falha de parsing
    xml_content = xml_content.sub(/\A\uFEFF/, '')

    doc = Nokogiri::XML(xml_content) do |config|
      config.strict.noblanks
    end

    # Validar que é XML bem formado
    unless doc.errors.empty?
      @errors << "XML mal formado: #{doc.errors.map(&:message).join(', ')}"
      return nil
    end

    doc
  rescue Nokogiri::XML::SyntaxError => e
    @errors << "Erro ao processar XML: #{e.message} (Linha: #{e.line}, Coluna: #{e.column})"
    nil
  rescue Encoding::InvalidByteSequenceError => e
    @errors << "Erro de encoding: #{e.message}. Arquivo deve estar em UTF-8."
    nil
  end

  # Retorna o nó da mensagem semt (SctiesBalAcctgRpt) quando há wrapper PosicaoAtivosCarteira,
  # como no fluxo Performit (Grails/Rails/abstract-anbima5). Caso contrário, retorna o próprio doc.
  def resolve_message_root(doc)
    root = doc.root
    return doc unless root

    local = root.name.to_s
    # Wrapper ANBIMA: PosicaoAtivosCarteira (com ou sem prefixo) > Document > SctiesBalAcctgRpt
    if local == 'PosicaoAtivosCarteira'
      # Elemento pode estar no default namespace (semt) ou sem namespace após strip no Grails
      node = doc.at_xpath("//*[local-name()='SctiesBalAcctgRpt']") ||
             doc.at_xpath("//xmlns:SctiesBalAcctgRpt", xmlns: @namespace)
      return node if node
    end
    doc
  end

  def extract_bah(doc)
    # BAH pode vir como bsnsMsg (semt) ou AppHdr (head) — alinhado aos XMLs reais do sistema
    bah = doc.at_xpath("//xmlns:bsnsMsg", xmlns: @namespace)
    if bah
      return {
        informante: bah.at_xpath(".//xmlns:fr//xmlns:nm", xmlns: @namespace)&.text,
        cnpj_informante: bah.at_xpath(".//xmlns:fr//xmlns:id//xmlns:othr//xmlns:id", xmlns: @namespace)&.text,
        destinatario: bah.at_xpath(".//xmlns:to//xmlns:nm", xmlns: @namespace)&.text,
        msg_def_idr: bah.at_xpath(".//xmlns:msgDefIdr", xmlns: @namespace)&.text,
        biz_svc: bah.at_xpath(".//xmlns:bizSvc", xmlns: @namespace)&.text,
        id_msg_sender: bah.at_xpath(".//xmlns:galgoHdr//xmlns:idMsgSender", xmlns: @namespace)&.text
      }
    end
    # Header no namespace head (ex.: PosicaoAtivosCarteira com urn:AppHdr)
    hdr = doc.at_xpath("//urn:AppHdr", urn: @head_namespace) || doc.at_xpath("//*[local-name()='AppHdr']")
    return nil unless hdr

    {
      informante: hdr.at_xpath(".//*[local-name()='Fr']//*[local-name()='Nm']")&.text || hdr.at_xpath(".//urn:Fr//urn:Nm", urn: @head_namespace)&.text,
      cnpj_informante: hdr.at_xpath(".//*[local-name()='Fr']//*[local-name()='Othr']//*[local-name()='Id']")&.text || hdr.at_xpath(".//urn:Fr//urn:Id//urn:Othr//urn:Id", urn: @head_namespace)&.text,
      destinatario: hdr.at_xpath(".//*[local-name()='To']//*[local-name()='Nm']")&.text || hdr.at_xpath(".//urn:To//urn:Nm", urn: @head_namespace)&.text,
      msg_def_idr: hdr.at_xpath(".//*[local-name()='MsgDefIdr']")&.text || hdr.at_xpath(".//urn:MsgDefIdr", urn: @head_namespace)&.text,
      biz_svc: hdr.at_xpath(".//*[local-name()='BizSvc']")&.text || hdr.at_xpath(".//urn:BizSvc", urn: @head_namespace)&.text,
      id_msg_sender: nil
    }
  end

  def extract_pagination(doc)
    pgntn = doc.at_xpath("//xmlns:Pgntn", xmlns: @namespace)
    return nil unless pgntn

    {
      pagina_atual: pgntn.at_xpath(".//xmlns:PgNb", xmlns: @namespace)&.text,
      ultima_pagina: pgntn.at_xpath(".//xmlns:LastPgInd", xmlns: @namespace)&.text == 'true'
    }
  end

  def extract_providers(doc)
    {
      administrador: extract_provider(doc, 'AcctOwnr'),
      gestor: extract_provider(doc, 'AcctSvcr'),
      custodiante: extract_provider(doc, 'SfkpgAcct')
    }
  end

  def extract_provider(doc, element_name)
    provider = doc.at_xpath("//xmlns:#{element_name}", xmlns: @namespace)
    return nil unless provider

    {
      cnpj: provider.at_xpath(".//xmlns:id//xmlns:othr//xmlns:id", xmlns: @namespace)&.text,
      nome: provider.at_xpath(".//xmlns:nm", xmlns: @namespace)&.text
    }
  end

  def extract_general_details(doc)
    stmt_gnl_dtls = doc.at_xpath("//xmlns:StmtGnlDtls", xmlns: @namespace)
    return nil unless stmt_gnl_dtls

    {
      data_posicao: stmt_gnl_dtls.at_xpath(".//xmlns:FrDtToDt//xmlns:FrDt", xmlns: @namespace)&.text,
      operacao: stmt_gnl_dtls.at_xpath(".//xmlns:QryRef", xmlns: @namespace)&.text,
      frequencia: stmt_gnl_dtls.at_xpath(".//xmlns:frqcy//xmlns:cd", xmlns: @namespace)&.text,
      tipo_atualizacao: stmt_gnl_dtls.at_xpath(".//xmlns:updTp//xmlns:cd", xmlns: @namespace)&.text
    }
  end

  def extract_portfolio_details(doc)
    doc.xpath("//xmlns:BalForAcct", xmlns: @namespace).map do |bal|
      {
        isin: bal.at_xpath(".//xmlns:FinInstrmId//xmlns:ISIN", xmlns: @namespace)&.text,
        cnpj: bal.at_xpath(".//xmlns:Id//xmlns:OrgId//xmlns:Othr//xmlns:Id", xmlns: @namespace)&.text,
        quantidade_cotas: bal.at_xpath(".//xmlns:Bal//xmlns:Qty", xmlns: @namespace)&.text,
        valor_cota: bal.at_xpath(".//xmlns:Bal//xmlns:Valtn//xmlns:Amt", xmlns: @namespace)&.text,
        total_ativos: bal.at_xpath(".//xmlns:Bal//xmlns:Valtn//xmlns:Amt", xmlns: @namespace)&.text
      }
    end
  end

  def extract_assets(doc)
    doc.xpath("//xmlns:SubAcctDtls", xmlns: @namespace).map do |asset|
      {
        isin: asset.at_xpath(".//xmlns:FinInstrmId//xmlns:ISIN", xmlns: @namespace)&.text,
        nome: asset.at_xpath(".//xmlns:FinInstrmAttrbts//xmlns:Nm", xmlns: @namespace)&.text,
        quantidade: asset.at_xpath(".//xmlns:Bal//xmlns:Qty", xmlns: @namespace)&.text,
        valor: asset.at_xpath(".//xmlns:Bal//xmlns:Valtn//xmlns:Amt", xmlns: @namespace)&.text,
        tipo: asset.at_xpath(".//xmlns:FinInstrmAttrbts//xmlns:ClssfctnTp", xmlns: @namespace)&.text
      }
    end
  end

  def extract_expenses(doc)
    expenses = []

    # Despesas liquidadas
    doc.xpath("//xmlns:Bal//xmlns:BalTp[.='EXPN']", xmlns: @namespace).each do |expense|
      expenses << {
        tipo: 'EXPN',
        valor: expense.at_xpath("../xmlns:Valtn//xmlns:Amt", xmlns: @namespace)&.text,
        descricao: 'Despesas liquidadas'
      }
    end

    # Outras despesas (MANF, EQUL, CUST, etc.)
    %w[MANF EQUL CUST BRKF TAXS OTHR].each do |code|
      doc.xpath("//xmlns:Bal//xmlns:BalTp[.='#{code}']", xmlns: @namespace).each do |expense|
        expenses << {
          tipo: code,
          valor: expense.at_xpath("../xmlns:Valtn//xmlns:Amt", xmlns: @namespace)&.text,
          descricao: expense_description(code)
        }
      end
    end

    expenses
  end

  def expense_description(code)
    descriptions = {
      'MANF' => 'Taxa de Administração',
      'EQUL' => 'Taxa de Performance',
      'CUST' => 'Taxa de Custódia',
      'BRKF' => 'Corretagem',
      'TAXS' => 'Tributos',
      'OTHR' => 'Outras Despesas'
    }
    descriptions[code] || code
  end

  # Validação alinhada ao fluxo Performit: Grails (Anbima5ImportService), Rails (Anbima5ReaderService),
  # frontend (AbstractAnbima5Service). Aceita wrapper PosicaoAtivosCarteira > Document > SctiesBalAcctgRpt
  # e exige os elementos que o sistema efetivamente usa para leitura ANBIMA5.
  def validate_structure(doc)
    errors = []

    root = doc.root
    root_local = root&.name&.to_s

    # Wrapper ANBIMA5: PosicaoAtivosCarteira (sem namespace ou com qualquer um) — como no Grails accept()
    if root_local == 'PosicaoAtivosCarteira'
      # Não exige namespace no root; o sistema faz strip de prefixos antes de parsear
      unless doc.at_xpath("//*[local-name()='SctiesBalAcctgRpt']") || doc.at_xpath("//xmlns:SctiesBalAcctgRpt", xmlns: @namespace)
        errors << "Mensagem SctiesBalAcctgRpt não encontrada (esperado dentro de Document no wrapper PosicaoAtivosCarteira)"
      end
    else
      # Raiz direta: exige namespace semt (semt.003.001.04)
      unless root&.namespace&.href == @namespace
        errors << "Namespace incorreto. Esperado: #{@namespace}, Encontrado: #{root&.namespace&.href}"
      end
    end

    # Elementos que o sistema usa na leitura (Grails/Rails/frontend)
    errors << "BalForAcct não encontrado" unless doc.at_xpath("//xmlns:BalForAcct", xmlns: @namespace) || doc.at_xpath("//*[local-name()='BalForAcct']")
    errors << "StmtGnlDtls (detalhes gerais/data posição) não encontrado" unless doc.at_xpath("//xmlns:StmtGnlDtls", xmlns: @namespace) || doc.at_xpath("//*[local-name()='StmtGnlDtls']")
    errors << "AcctBaseCcyTtlAmts (PL total) não encontrado" unless doc.at_xpath("//xmlns:AcctBaseCcyTtlAmts", xmlns: @namespace) || doc.at_xpath("//*[local-name()='AcctBaseCcyTtlAmts']")

    # Conformidade ANBIMA5 (opcionais para importação, obrigatórios para envio)
    errors << "BAH (bsnsMsg) não encontrado" unless doc.at_xpath("//xmlns:bsnsMsg", xmlns: @namespace) || doc.at_xpath("//*[local-name()='bsnsMsg']")
    errors << "Paginação (Pgntn) não encontrada" unless doc.at_xpath("//xmlns:Pgntn", xmlns: @namespace) || doc.at_xpath("//*[local-name()='Pgntn']")
    errors << "Custodiante (SfkpgAcct) não encontrado" unless doc.at_xpath("//xmlns:SfkpgAcct", xmlns: @namespace) || doc.at_xpath("//*[local-name()='SfkpgAcct']")

    @structure_errors = errors
    errors.empty?
  end

  def validate_required_fields(doc)
    errors = []

    # Campos 001-008 (BAH) — busca com namespace ou local-name (wrapper ANBIMA5)
    bah = doc.at_xpath("//xmlns:bsnsMsg", xmlns: @namespace) || doc.at_xpath("//*[local-name()='bsnsMsg']")
    if bah
      errors << "Campo 001: Informante não encontrado" unless bah.at_xpath(".//xmlns:fr//xmlns:nm", xmlns: @namespace)
      errors << "Campo 002: CNPJ informante não encontrado" unless bah.at_xpath(".//xmlns:fr//xmlns:id//xmlns:othr//xmlns:id", xmlns: @namespace)

      # Validar formato CNPJ (14 caracteres)
      cnpj = bah.at_xpath(".//xmlns:fr//xmlns:id//xmlns:othr//xmlns:id", xmlns: @namespace)&.text
      if cnpj && cnpj.length != 14
        errors << "Campo 002: CNPJ informante deve ter 14 caracteres (encontrado: #{cnpj.length})"
      end

      # Validar mensagem
      msg_def_idr = bah.at_xpath(".//xmlns:msgDefIdr", xmlns: @namespace)&.text
      unless msg_def_idr == 'semt.003.001.04'
        errors << "Campo 005: Mensagem deve ser 'semt.003.001.04' (encontrado: #{msg_def_idr})"
      end

      # Validar serviço
      biz_svc = bah.at_xpath(".//xmlns:bizSvc", xmlns: @namespace)&.text
      unless biz_svc == 'Arquivo de Posição 5.0'
        errors << "Campo 006: Serviço deve ser 'Arquivo de Posição 5.0' (encontrado: #{biz_svc})"
      end
    end

    # Campos 009-016 (Paginação e Status) — StmtGnlDtls/FrDtToDt/FrDt (data posição, usado pelo Grails)
    stmt_dt = doc.at_xpath("//xmlns:StmtGnlDtls//xmlns:FrDtToDt//xmlns:FrDt", xmlns: @namespace) ||
              doc.at_xpath("//*[local-name()='StmtGnlDtls']//*[local-name()='FrDt']")
    errors << "Campo 013: Data posição (StmtGnlDtls/FrDtToDt/FrDt) não encontrada" unless stmt_dt

    errors
  end

  # Cálculos alinhados ao fluxo Performit: PL em AcctBaseCcyTtlAmts/TtlHldgsValOfStmt (frontend netWorth),
  # quantidade e preço no primeiro BalForAcct (Grails: AggtBal/Qty/Unit, PricDtls/Val/Amt).
  def validate_calculations(doc)
    errors = []

    # PL declarado (como no frontend netWorth e Grails AcctBaseCcyTtlAmts)
    ttl_pl = doc.at_xpath("//xmlns:AcctBaseCcyTtlAmts//xmlns:TtlHldgsValOfStmt", xmlns: @namespace) ||
             doc.at_xpath("//*[local-name()='AcctBaseCcyTtlAmts']//*[local-name()='TtlHldgsValOfStmt']")
    unless ttl_pl
      errors << "PL total (AcctBaseCcyTtlAmts/TtlHldgsValOfStmt) não encontrado"
      return errors
    end

    amt_elem = ttl_pl.at_xpath(".//xmlns:Amt", xmlns: @namespace) || ttl_pl.at_xpath(".//*[local-name()='Amt']")
    sgn_elem = ttl_pl.at_xpath(".//xmlns:Sgn", xmlns: @namespace) || ttl_pl.at_xpath(".//*[local-name()='Sgn']")
    unless amt_elem
      errors << "Valor do PL (TtlHldgsValOfStmt/Amt) não encontrado"
      return errors
    end

    pl_declarado = amt_elem.text.to_s.tr(',', '.').to_f
    pl_declarado *= -1 if sgn_elem&.text == 'false'

    # Primeiro BalForAcct: quantidade (AggtBal) e preço (PricDtls) — como no Grails readLines/administratorPortfolioSave
    first_bal = doc.at_xpath("//xmlns:BalForAcct", xmlns: @namespace) || doc.at_xpath("//*[local-name()='BalForAcct']")
    if first_bal
      qty_path = ".//xmlns:AggtBal//xmlns:Qty//xmlns:Qty//xmlns:Qty//xmlns:Unit"
      qty_elem = first_bal.at_xpath(qty_path, xmlns: @namespace) || first_bal.at_xpath(".//*[local-name()='AggtBal']//*[local-name()='Qty']//*[local-name()='Unit']")
      pric_path = ".//xmlns:PricDtls//xmlns:Val//xmlns:Amt"
      pric_elem = first_bal.at_xpath(pric_path, xmlns: @namespace) || first_bal.at_xpath(".//*[local-name()='PricDtls']//*[local-name()='Val']//*[local-name()='Amt']")
      if qty_elem && pric_elem
        quantidade = qty_elem.text.to_s.tr(',', '.').to_f
        valor_cota = pric_elem.text.to_s.tr(',', '.').to_f
        pl_calculado = quantidade * valor_cota
        diferenca = (pl_calculado - pl_declarado).abs
        if diferenca >= 0.01
          errors << "Divergência PL: Cotas×ValorCota=#{pl_calculado}, TtlHldgsValOfStmt=#{pl_declarado}, Diferença=#{diferenca.round(4)}"
        end
      end
    end

    errors
  end

  def filter_sections(analysis, sections)
    filtered = {}
    section_map = {
      'BalForAcct' => :carteira,
      'SubAcctDtls' => :ativos,
      'EXPN' => :despesas,
      'bsnsMsg' => :bah,
      'Pgntn' => :paginacao,
      'StmtGnlDtls' => :detalhes_gerais
    }

    sections.each do |section|
      key = section_map[section] || section.to_sym
      filtered[key] = analysis[key] if analysis.key?(key)
    end

    # Sempre incluir validação se solicitada
    filtered[:validacao] = analysis[:validacao] if @validate

    filtered
  end

  def format_output(analysis)
    case @format
    when 'json'
      format_json(analysis)
    when 'detailed'
      format_detailed(analysis)
    else
      format_summary(analysis)
    end
  end

  def format_summary(analysis)
    output = []
    output << "## 📋 Análise do Arquivo XML5 ANBIMA"
    output << ""

    if analysis[:bah]
      output << "### Informações Gerais"
      output << "- Informante: #{analysis[:bah][:informante]}"
      output << "- CNPJ: #{analysis[:bah][:cnpj_informante]}"
      output << "- Data Posição: #{analysis[:detalhes_gerais]&.dig(:data_posicao) || 'N/A'}"
      output << ""
    end

    if analysis[:carteira] && !analysis[:carteira].empty?
      output << "### Carteira"
      output << "- Total de Ativos: #{analysis[:carteira].first[:total_ativos] || 'N/A'}"
      output << "- Quantidade de Cotas: #{analysis[:carteira].first[:quantidade_cotas] || 'N/A'}"
      output << "- Valor da Cota: #{analysis[:carteira].first[:valor_cota] || 'N/A'}"
      output << ""
    end

    if analysis[:validacao]
      output << "### Validações"
      estrutura_ok = analysis[:validacao][:estrutura]
      campos_ok = analysis[:validacao][:campos_obrigatorios].empty?
      calculos_ok = analysis[:validacao][:calculos].empty?

      output << "- Estrutura: #{estrutura_ok ? '✅ Válida' : '❌ Inválida'}"
      output << "- Campos Obrigatórios: #{campos_ok ? '✅ OK' : "❌ #{analysis[:validacao][:campos_obrigatorios].count} erros"}"
      output << "- Cálculos: #{calculos_ok ? '✅ OK' : "❌ #{analysis[:validacao][:calculos].count} erros"}"

      unless estrutura_ok
        output << ""
        output << "#### Erros de Estrutura (conformidade ANBIMA5 / uso no Performit):"
        (analysis[:validacao][:estrutura_erros] || []).each { |err| output << "  - #{err}" }
      end

      unless campos_ok
        output << ""
        output << "#### Erros de Campos Obrigatórios:"
        analysis[:validacao][:campos_obrigatorios].each { |error| output << "  - #{error}" }
      end

      unless calculos_ok
        output << ""
        output << "#### Erros de Cálculos:"
        analysis[:validacao][:calculos].each { |error| output << "  - #{error}" }
      end
    end

    output.join("\n")
  end

  def format_detailed(analysis)
    JSON.pretty_generate(analysis)
  end

  def format_json(analysis)
    JSON.pretty_generate(analysis)
  end
end

# Parsing de argumentos da linha de comando
options = {
  validate: true,
  format: 'summary'
}

OptionParser.new do |opts|
  opts.banner = "Uso: read-xml5-anbima.rb [opções]"

  opts.on("--file=FILE", "Caminho do arquivo XML (obrigatório)") do |file|
    options[:file] = file
  end

  opts.on("--validate=[true|false]", "Executar validação completa (padrão: true)") do |validate|
    options[:validate] = validate != 'false'
  end

  opts.on("--extract=SECTIONS", "Extrair seções específicas (ex: BalForAcct,SubAcctDtls)") do |extract|
    options[:extract] = extract
  end

  opts.on("--format=FORMAT", "Formato de saída: summary, detailed, json (padrão: summary)") do |format|
    options[:format] = format
  end

  opts.on("-h", "--help", "Mostrar esta ajuda") do
    puts opts
    exit
  end
end.parse!

# Validar argumentos obrigatórios
unless options[:file]
  puts "Erro: --file é obrigatório"
  puts "Use --help para ver opções disponíveis"
  exit 1
end

# Executar análise
begin
  analyzer = XML5Analyzer.new(
    options[:file],
    validate: options[:validate],
    extract: options[:extract],
    format: options[:format]
  )

  result = analyzer.analyze
  if result
    puts result
  else
    puts "Erro ao analisar arquivo XML"
    exit 1
  end
rescue StandardError => e
  puts "❌ Erro: #{e.message}"
  exit 1
end
