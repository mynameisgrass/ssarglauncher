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
