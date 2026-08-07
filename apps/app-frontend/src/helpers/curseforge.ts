import { invoke } from '@tauri-apps/api/core'

export interface CurseForgeSearchItem {
	id: number
	gameId: number
	name: string
	slug: string
	summary: string
	downloadCount: number
	logo?: {
		id: number
		url: string
		thumbnailUrl: string
	}
	authors: Array<{ id: number; name: string; url: string }>
	categories: Array<{ id: number; name: string; slug: string }>
	latestFiles: Array<{
		id: number
		displayName: string
		fileName: string
		downloadUrl: string
		gameVersions: string[]
	}>
}

export function getCurseForgeClassId(projectType: string): number {
	switch (projectType) {
		case 'modpack':
			return 4471
		case 'mod':
			return 6
		case 'resourcepack':
			return 12
		case 'shader':
			return 6552
		default:
			return 4471
	}
}

export async function searchCurseForge(
	query: string,
	projectType: string = 'modpack',
	pageSize: number = 20,
	index: number = 0,
): Promise<CurseForgeSearchItem[]> {
	try {
		const classId = getCurseForgeClassId(projectType)
		console.group(`[CurseForge IPC Search] Query: "${query}" | Type: ${projectType}`)

		const json = await invoke<any>('search_curseforge', {
			query,
			classId,
			pageSize,
			index,
		})

		const items: CurseForgeSearchItem[] = json?.data ?? []
		console.log(`[CurseForge IPC Success] Received ${items.length} items:`, items.map((i) => ({ id: i.id, name: i.name, slug: i.slug })))
		console.groupEnd()

		return items
	} catch (err) {
		console.error('[CurseForge IPC Search Error]:', err)
		console.groupEnd()
		return []
	}
}

export async function getCurseForgeProject(id: string) {
	try {
		const json = await invoke<any>('get_curseforge_project', { id })
		const cfItem = json?.data
		if (!cfItem) return null

		let projectType = 'mod'
		if (cfItem.classId === 4471) projectType = 'modpack'
		else if (cfItem.classId === 12) projectType = 'resourcepack'
		else if (cfItem.classId === 6552) projectType = 'shader'

		return {
			id: `cf_${cfItem.id}`,
			slug: cfItem.slug,
			project_type: projectType,
			team: `cf_team_${cfItem.id}`,
			organization: null,
			title: cfItem.name,
			description: cfItem.summary,
			body: cfItem.summary,
			published: cfItem.dateCreated,
			updated: cfItem.dateModified,
			approved: cfItem.dateReleased,
			status: 'approved',
			license: {
				id: 'CurseForge',
				name: 'CurseForge License',
				url: null,
			},
			client_side: 'optional',
			server_side: 'optional',
			downloads: cfItem.downloadCount,
			followers: 0,
			categories: ['curseforge', ...(cfItem.categories?.map((c: any) => c.slug) ?? [])],
			additional_categories: [],
			game_versions: [],
			loaders: [],
			versions: [],
			icon_url: cfItem.logo?.thumbnailUrl ?? cfItem.logo?.url ?? '',
			issues_url: null,
			source_url: null,
			wiki_url: null,
			discord_url: null,
			donation_urls: [],
			gallery: cfItem.screenshots?.map((s: any) => ({
				url: s.url,
				featured: false,
				title: s.title,
				description: s.description,
				created: new Date().toISOString(),
				ordering: 0,
			})) ?? [],
			color: null,
		}
	} catch (err) {
		console.error('[CurseForge IPC Project Error]:', err)
		return null
	}
}

export async function getCurseForgeProjectVersions(id: string) {
	try {
		const json = await invoke<any>('get_curseforge_project_versions', { id })
		const files = json?.data
		if (!Array.isArray(files)) return []

		return files.map((file: any) => ({
			id: `cf_${file.id}`,
			project_id: `cf_${file.modId}`,
			author_id: 'curseforge',
			featured: true,
			name: file.displayName,
			version_number: file.displayName,
			changelog: '',
			changelog_url: null,
			date_published: file.fileDate,
			downloads: file.downloadCount,
			version_type: file.releaseType === 1 ? 'release' : file.releaseType === 2 ? 'beta' : 'alpha',
			status: 'listed',
			requested_status: 'listed',
			files: [
				{
					hashes: {
						sha1: file.hashes?.find((h: any) => h.algo === 1)?.value ?? '',
						sha512: '',
					},
					url: file.downloadUrl,
					filename: file.fileName,
					primary: true,
					size: file.fileLength,
					file_type: null,
				}
			],
			dependencies: file.dependencies?.map((d: any) => ({
				version_id: null,
				project_id: `cf_${d.modId}`,
				file_name: null,
				dependency_type: d.relationType === 3 ? 'required' : 'optional',
			})) ?? [],
			game_versions: file.gameVersions ?? [],
			loaders: file.gameVersions ?? [],
		}))
	} catch (err) {
		console.error('[CurseForge IPC Versions Error]:', err)
		return []
	}
}
