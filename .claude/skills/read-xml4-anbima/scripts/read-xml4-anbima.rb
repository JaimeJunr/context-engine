#!/usr/bin/env ruby
# frozen_string_literal: true

# read-xml4-anbima.rb - Script da skill read-xml4-anbima: leitura e análise de arquivos XML4 ANBIMA
# Alinhado ao fluxo Performit: AnbimaImportService (Grails), AbstractAnbimaService (frontend)

require 'nokogiri'
require 'json'
require 'optparse'
require 'fileutils'

class XML4Analyzer
  def initialize(file_path, validate: true, extract: nil, format: 'summary')
    @file_path = file_path
    @validate = validate
    @extract_sections = extract&.split(',')&.map(&:strip)
    @format = format
    @errors = []
    @structure_errors = []
  end

  def analyze
    validate_file_exists
    doc = load_and_parse_xml
    return nil unless doc

    analysis = {
      header: extract_header(doc),
      patliq_root: extract_patliq_root(doc),
      provisao: extract_provisao(doc),
      caixa: extract_caixa(doc),
      validacao: {
        estrutura: validate_structure(doc),
        estrutura_erros: @structure_errors || [],
        campos_obrigatorios: validate_required_fields(doc),
        calculos: validate_calculations(doc)
      }
    }

    analysis = filter_sections(analysis, @extract_sections) if @extract_sections
    format_output(analysis)
  end

  private

  def validate_file_exists
    raise "Arquivo não encontrado: #{@file_path}" unless File.exist?(@file_path)
    raise "Arquivo não pode ser lido: #{@file_path}" unless File.readable?(@file_path)
  end

  def load_and_parse_xml
    content = File.read(@file_path, encoding: 'UTF-8')
    content = content.sub(/\A\uFEFF/, '')
    doc = Nokogiri::XML(content) { |c| c.strict.noblanks }
    unless doc.errors.empty?
      @errors << "XML mal formado: #{doc.errors.map(&:message).join(', ')}"
      return nil
    end
    doc
  rescue Nokogiri::XML::SyntaxError => e
    @errors << "Erro ao processar XML: #{e.message} (Linha: #{e.line})"
    nil
  rescue Encoding::InvalidByteSequenceError => e
    @errors << "Erro de encoding: #{e.message}. Use UTF-8 ou declare encoding no XML."
    nil
  end

  def header_node(doc)
    root = doc.root
    return nil unless root
    fundo = root.at_css('fundo')
    carteira = root.at_css('carteira')
    (fundo || carteira)&.at_css('header')
  end

  def extract_header(doc)
    h = header_node(doc)
    return nil unless h
    {
      nome: h.at_css('nome')&.text,
      dtposicao: h.at_css('dtposicao')&.text,
      patliq: h.at_css('patliq')&.text,
      quantidade: h.at_css('quantidade')&.text,
      valorcota: h.at_css('valorcota')&.text
    }
  end

  def extract_patliq_root(doc)
    root = doc.root
    return {} unless root
    {
      patliq: root.at_css('patliq')&.text,
      vlcotasemitir: root.at_css('vlcotasemitir')&.text,
      vlcotasresgatar: root.at_css('vlcotasresgatar')&.text
    }
  end

  def extract_provisao(doc)
    doc.css('provisao').map do |p|
      cod = p.at_css('coddespesa')&.text
      cod = p.at_css('codprov')&.text if cod.to_s.empty?
      {
        cod: cod,
        valor: p.at_css('valor')&.text,
        credeb: p.at_css('credeb')&.text,
        dt: p.at_css('dt')&.text
      }
    end
  end

  def extract_caixa(doc)
    doc.css('caixa').map do |c|
      {
        isininstituicao: c.at_css('isininstituicao')&.text,
        saldo: c.at_css('saldo')&.text
      }
    end
  end

  def validate_structure(doc)
    @structure_errors = []
    root = doc.root
    unless root
      @structure_errors << "Documento sem elemento raiz"
      return false
    end
    unless root.name.to_s.include?('arquivoposicao_4')
      @structure_errors << "Raiz deve ser arquivoposicao_4_01 (encontrado: #{root.name})"
    end
    h = header_node(doc)
    @structure_errors << "Header não encontrado (esperado fundo/header ou carteira/header)" unless h
    @structure_errors.empty?
  end

  def validate_required_fields(doc)
    errors = []
    h = header_node(doc)
    unless h
      errors << "Header não disponível para validar campos"
      return errors
    end
    errors << "Campo nome não encontrado" unless h.at_css('nome')&.text.to_s.strip != ''
    dt = h.at_css('dtposicao')&.text
    errors << "Campo dtposicao não encontrado ou vazio" if dt.to_s.strip.empty?
    errors << "Campo patliq não encontrado" unless h.at_css('patliq')
    errors << "Campo quantidade não encontrado" unless h.at_css('quantidade')
    errors << "Campo valorcota não encontrado" unless h.at_css('valorcota')
    errors
  end

  def validate_calculations(doc)
    errors = []
    h = header_node(doc)
    return errors unless h
    patliq = h.at_css('patliq')&.text.to_s.tr(',', '.').to_f
    qty = h.at_css('quantidade')&.text.to_s.tr(',', '.').to_f
    price = h.at_css('valorcota')&.text.to_s.tr(',', '.').to_f
    return errors if qty.zero? || price.zero?
    pl_calc = qty * price
    diff = (pl_calc - patliq).abs
    errors << "Divergência PL: quantidade×valorcota=#{pl_calc}, patliq=#{patliq}, Diferença=#{diff.round(4)}" if diff >= 0.01
    errors
  end

  def filter_sections(analysis, sections)
    section_map = {
      'header' => :header,
      'provisao' => :provisao,
      'caixa' => :caixa,
      'patliq_root' => :patliq_root
    }
    filtered = {}
    sections.each do |s|
      key = section_map[s] || s.to_sym
      filtered[key] = analysis[key] if analysis.key?(key)
    end
    filtered[:validacao] = analysis[:validacao] if @validate
    filtered
  end

  def format_output(analysis)
    case @format
    when 'json' then JSON.pretty_generate(analysis)
    when 'detailed' then JSON.pretty_generate(analysis)
    else format_summary(analysis)
    end
  end

  def format_summary(analysis)
    out = []
    out << "## 📋 Análise do Arquivo XML4 ANBIMA"
    out << ""
    if analysis[:header]
      out << "### Header (fundo/carteira)"
      out << "- Nome: #{analysis[:header][:nome]}"
      out << "- Data posição: #{analysis[:header][:dtposicao]}"
      out << "- PL (patliq): #{analysis[:header][:patliq]}"
      out << "- Quantidade: #{analysis[:header][:quantidade]}"
      out << "- Valor cota: #{analysis[:header][:valorcota]}"
      out << ""
    end
    if analysis[:patliq_root] && (analysis[:patliq_root][:vlcotasemitir] || analysis[:patliq_root][:vlcotasresgatar])
      out << "### Cotas (raiz)"
      out << "- VL cotas emitir: #{analysis[:patliq_root][:vlcotasemitir]}"
      out << "- VL cotas resgatar: #{analysis[:patliq_root][:vlcotasresgatar]}"
      out << ""
    end
    if analysis[:validacao]
      v = analysis[:validacao]
      est_ok = v[:estrutura]
      camp_ok = v[:campos_obrigatorios].empty?
      calc_ok = v[:calculos].empty?
      out << "### Validações"
      out << "- Estrutura: #{est_ok ? '✅ Válida' : '❌ Inválida'}"
      out << "- Campos Obrigatórios: #{camp_ok ? '✅ OK' : "❌ #{v[:campos_obrigatorios].count} erros"}"
      out << "- Cálculos: #{calc_ok ? '✅ OK' : "❌ #{v[:calculos].count} erros"}"
      unless est_ok
        out << ""
        out << "#### Erros de Estrutura:"
        (v[:estrutura_erros] || []).each { |e| out << "  - #{e}" }
      end
      unless camp_ok
        out << ""
        out << "#### Erros de Campos:"
        v[:campos_obrigatorios].each { |e| out << "  - #{e}" }
      end
      unless calc_ok
        out << ""
        out << "#### Erros de Cálculos:"
        v[:calculos].each { |e| out << "  - #{e}" }
      end
    end
    out.join("\n")
  end
end

options = { validate: true, format: 'summary' }
OptionParser.new do |opts|
  opts.banner = "Uso: read-xml4-anbima.rb [opções]"
  opts.on("--file=FILE", "Caminho do arquivo XML (obrigatório)") { |f| options[:file] = f }
  opts.on("--validate=[true|false]", "Executar validação (padrão: true)") { |v| options[:validate] = v != 'false' }
  opts.on("--extract=SECTIONS", "Seções: header,provisao,caixa,...") { |e| options[:extract] = e }
  opts.on("--format=FORMAT", "summary|detailed|json (padrão: summary)") { |f| options[:format] = f }
  opts.on("-h", "--help", "Ajuda") { puts opts; exit }
end.parse!

unless options[:file]
  puts "Erro: --file é obrigatório"
  exit 1
end

begin
  analyzer = XML4Analyzer.new(options[:file], validate: options[:validate], extract: options[:extract], format: options[:format])
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
