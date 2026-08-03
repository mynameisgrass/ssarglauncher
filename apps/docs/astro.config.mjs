import starlight from '@astrojs/starlight'
import { defineConfig } from 'astro/config'
import starlightOpenAPI, { openAPISidebarGroups } from 'starlight-openapi'

// https://astro.build/config
export default defineConfig({
	site: 'https://launcher.grassist.me',
	integrations: [
		starlight({
			title: 'Ssarg Launcher Documentation',
			favicon: '/favicon.ico',
			editLink: {
				baseUrl: 'https://github.com/mynameisgrass/ssarglauncher/edit/main/apps/docs/',
			},
			social: {
				github: 'https://github.com/mynameisgrass/ssarglauncher',
			},
			logo: {
				light: './src/assets/light-logo.svg',
				dark: './src/assets/dark-logo.svg',
				replacesTitle: false,
			},
			customCss: [
				'@modrinth/assets/styles/variables.scss',
				'@modrinth/assets/styles/inter.scss',
				'./src/styles/modrinth.css',
			],
			plugins: [
				// Generate the OpenAPI documentation pages.
				starlightOpenAPI([
					{
						base: 'api',
						label: 'Modrinth API',
						schema: './public/openapi.yaml',
					},
				]),
			],
			sidebar: [
				{
					label: 'Contributing to Ssarg Launcher',
					autogenerate: { directory: 'contributing' },
				},
				{
					label: 'Guides',
					autogenerate: { directory: 'guide' },
				},
				// Add the generated sidebar group to the sidebar.
				...openAPISidebarGroups,
			],
		}),
	],
})
