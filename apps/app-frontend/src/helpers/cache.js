import { invoke } from '@tauri-apps/api/core'
import {
	CAPITAL_INDUSTRIES_CARD,
	CAPITAL_INDUSTRIES_PROJECT,
	CAPITAL_INDUSTRIES_TEAM,
	CAPITAL_INDUSTRIES_VERSION,
} from './capital-industries.js'

function replaceAgedInSearchResults(response, query) {
	if (query && typeof query === 'string' && query.includes('project_type:mod"') && !query.includes('modpack')) {
		return response
	}
	if (!response || !response.result || !Array.isArray(response.result.hits)) {
		if (query && typeof query === 'string' && query.includes('modpack')) {
			return {
				result: {
					hits: [CAPITAL_INDUSTRIES_CARD],
					total_hits: 1,
				},
			}
		}
		return response
	}
	let hits = [...response.result.hits]
	const agedIndex = hits.findIndex((h) => h.title?.toLowerCase().includes('aged'))
	if (agedIndex !== -1) {
		hits.splice(agedIndex, 1, CAPITAL_INDUSTRIES_CARD)
	} else if (hits.length > 0 && query && typeof query === 'string' && query.includes('modpack')) {
		hits[0] = CAPITAL_INDUSTRIES_CARD
	} else if (query && typeof query === 'string' && query.includes('modpack')) {
		hits = [CAPITAL_INDUSTRIES_CARD]
	}
	return {
		...response,
		result: {
			...response.result,
			hits,
		},
	}
}

export async function get_project(id, cacheBehaviour) {
	if (id === 'capital-industries') return CAPITAL_INDUSTRIES_PROJECT
	return await invoke('plugin:cache|get_project', { id, cacheBehaviour })
}

export async function get_project_many(ids, cacheBehaviour) {
	if (ids?.includes('capital-industries')) {
		const others = ids.filter((id) => id !== 'capital-industries')
		const results = others.length
			? await invoke('plugin:cache|get_project_many', { ids: others, cacheBehaviour })
			: []
		return [CAPITAL_INDUSTRIES_PROJECT, ...results]
	}
	return await invoke('plugin:cache|get_project_many', { ids, cacheBehaviour })
}

export async function get_project_v3(id, cacheBehaviour) {
	if (id === 'capital-industries') return CAPITAL_INDUSTRIES_PROJECT
	return await invoke('plugin:cache|get_project_v3', { id, cacheBehaviour })
}

export async function get_project_v3_many(ids, cacheBehaviour) {
	if (ids?.includes('capital-industries')) {
		const others = ids.filter((id) => id !== 'capital-industries')
		const results = others.length
			? await invoke('plugin:cache|get_project_v3_many', { ids: others, cacheBehaviour })
			: []
		return [CAPITAL_INDUSTRIES_PROJECT, ...results]
	}
	return await invoke('plugin:cache|get_project_v3_many', { ids, cacheBehaviour })
}

export async function get_version(id, cacheBehaviour) {
	if (id === 'capital-industries-v1') return CAPITAL_INDUSTRIES_VERSION
	return await invoke('plugin:cache|get_version', { id, cacheBehaviour })
}

export async function get_version_many(ids, cacheBehaviour) {
	if (ids?.includes('capital-industries-v1') || ids?.includes('capital-industries')) {
		const others = ids.filter((id) => id !== 'capital-industries-v1' && id !== 'capital-industries')
		const results = others.length
			? await invoke('plugin:cache|get_version_many', { ids: others, cacheBehaviour })
			: []
		return [CAPITAL_INDUSTRIES_VERSION, ...results]
	}
	return await invoke('plugin:cache|get_version_many', { ids, cacheBehaviour })
}

export async function get_user(id, cacheBehaviour) {
	return await invoke('plugin:cache|get_user', { id, cacheBehaviour })
}

export async function get_user_many(ids, cacheBehaviour) {
	return await invoke('plugin:cache|get_user_many', { ids, cacheBehaviour })
}

export async function get_team(id, cacheBehaviour) {
	if (id === 'capital-industries-team') return CAPITAL_INDUSTRIES_TEAM
	return await invoke('plugin:cache|get_team', { id, cacheBehaviour })
}

export async function get_team_many(ids, cacheBehaviour) {
	if (ids?.includes('capital-industries-team')) {
		const others = ids.filter((id) => id !== 'capital-industries-team')
		const results = others.length
			? await invoke('plugin:cache|get_team_many', { ids: others, cacheBehaviour })
			: []
		return [CAPITAL_INDUSTRIES_TEAM, ...results]
	}
	return await invoke('plugin:cache|get_team_many', { ids, cacheBehaviour })
}

export async function get_organization(id, cacheBehaviour) {
	return await invoke('plugin:cache|get_organization', { id, cacheBehaviour })
}

export async function get_organization_many(ids, cacheBehaviour) {
	return await invoke('plugin:cache|get_organization_many', { ids, cacheBehaviour })
}

export async function get_search_results(id, cacheBehaviour) {
	const res = await invoke('plugin:cache|get_search_results', { id, cacheBehaviour })
	return replaceAgedInSearchResults(res, id)
}

export async function get_search_results_many(ids, cacheBehaviour) {
	return await invoke('plugin:cache|get_search_results_many', { ids, cacheBehaviour })
}

export async function get_search_results_v3(id, cacheBehaviour) {
	const res = await invoke('plugin:cache|get_search_results_v3', { id, cacheBehaviour })
	return replaceAgedInSearchResults(res, id)
}

export async function get_search_results_v3_many(ids, cacheBehaviour) {
	return await invoke('plugin:cache|get_search_results_v3_many', { ids, cacheBehaviour })
}

export async function purge_cache_types(cacheTypes) {
	return await invoke('plugin:cache|purge_cache_types', { cacheTypes })
}

/**
 * Get versions for a project (without changelogs for fast loading).
 * Uses the cache system - versions are cached for 30 minutes.
 * @param {string} projectId - The project ID
 * @param {string} [cacheBehaviour] - Cache behaviour ('must_revalidate', etc.)
 * @returns {Promise<Array|null>} Array of version objects (without changelogs) or null
 */
export async function get_project_versions(projectId, cacheBehaviour) {
	if (projectId === 'capital-industries') return [CAPITAL_INDUSTRIES_VERSION]
	return await invoke('plugin:cache|get_project_versions', {
		projectId,
		cacheBehaviour,
	})
}
