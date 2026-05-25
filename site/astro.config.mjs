// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://jaimejunr.github.io',
	base: '/context-engine',
	integrations: [
		starlight({
			title: 'ctx',
			description:
				'All-in-one Rust toolkit for Claude Code & MCP agents: auto-rewrite hooks, MCP server, repo maps with BM25+PageRank, call graphs in 7 languages, local RAG. Single binary, 100% local.',
			logo: {
				src: './src/assets/logo.svg',
				replacesTitle: false,
			},
			social: [
				{
					icon: 'github',
					label: 'GitHub',
					href: 'https://github.com/JaimeJunr/context-engine',
				},
				{
					icon: 'seti:rust',
					label: 'crates.io',
					href: 'https://crates.io/crates/ctx-engine',
				},
			],
			editLink: {
				baseUrl: 'https://github.com/JaimeJunr/context-engine/edit/main/site/',
			},
			customCss: ['./src/styles/custom.css'],
			head: [
				{
					tag: 'meta',
					attrs: {
						property: 'og:image',
						content: 'https://jaimejunr.github.io/context-engine/og-image.png',
					},
				},
				{
					tag: 'meta',
					attrs: { name: 'twitter:card', content: 'summary_large_image' },
				},
			],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Install', slug: 'guides/install' },
						{ label: 'Quick Start', slug: 'guides/quick-start' },
						{ label: 'Agent integration', slug: 'guides/agent-integration' },
					],
				},
				{
					label: 'Subcommands',
					items: [
						{ label: 'ctx map', slug: 'subcommands/map' },
						{ label: 'ctx catalog', slug: 'subcommands/catalog' },
						{ label: 'ctx exec', slug: 'subcommands/exec' },
						{ label: 'ctx graph', slug: 'subcommands/graph' },
						{ label: 'ctx mcp', slug: 'subcommands/mcp' },
					],
				},
				{
					label: 'Compare',
					items: [{ label: 'vs RTK / CodeGraph / Context Mode / QMD', slug: 'compare' }],
				},
				{
					label: 'Architecture',
					items: [
						{ label: 'Modules', slug: 'architecture/modules' },
						{ label: 'Extending', slug: 'architecture/extending' },
					],
				},
			],
		}),
	],
});
