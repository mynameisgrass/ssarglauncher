import starlight from '@astrojs/starlight'
import { defineConfig } from 'astro/config'

// https://astro.build/config
export default defineConfig({
	site: 'https://launcher.grassist.me',
	integrations: [
		starlight({
			title: 'Ssarg Launcher',
			favicon: '/favicon.ico',
			editLink: {
				baseUrl: 'https://github.com/mynameisgrass/ssarglauncher/edit/main/apps/docs/',
			},
			social: {
				github: 'https://github.com/mynameisgrass/ssarglauncher',
			},
			logo: {
				src: './src/assets/logo.png',
				replacesTitle: false,
			},
			customCss: [
				'@modrinth/assets/styles/variables.scss',
				'@modrinth/assets/styles/inter.scss',
				'./src/styles/modrinth.css',
			],
			sidebar: [
				{
					label: 'Getting Started',
					autogenerate: { directory: 'getting-started' },
				},
				{
					label: 'Features',
					autogenerate: { directory: 'features' },
				},
			],
		}),
	],
})
