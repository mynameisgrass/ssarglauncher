<script setup lang="ts">
import { computed } from 'vue'

import { getTagMessage, sortTagsForDisplay } from '../../../utils'
import { TagTagItem } from '../../base'
import TagsOverflow from '../TagsOverflow.vue'

function isLoader(tag: string) {
	return getTagMessage(tag, 'loader') !== undefined
}

function uniqueSorted(tags?: string[]) {
	return tags ? sortTagsForDisplay([...new Set(tags)]) : undefined
}

const props = withDefaults(
	defineProps<{
		tags: string[]
		extraTags?: string[]
		deprioritizedTags?: string[]
		excludeLoaders?: boolean
		maxTags?: number
	}>(),
	{
		maxTags: 5,
		excludeLoaders: false,
		extraTags: () => [],
		deprioritizedTags: () => [],
	},
)

const sortedTags = computed(() => uniqueSorted(props.tags))
const sortedExtraTags = computed(() => uniqueSorted(props.extraTags))
const filteredTags = computed(() => {
	if (!sortedTags.value) {
		return undefined
	}
	const list = sortedTags.value.filter(
		(tag) => !props.deprioritizedTags.includes(tag) && (!props.excludeLoaders || !isLoader(tag)),
	)
	return list.sort((a, b) => {
		const isSourceA = ['curseforge', 'modrinth'].includes(a.toLowerCase()) ? 0 : 1
		const isSourceB = ['curseforge', 'modrinth'].includes(b.toLowerCase()) ? 0 : 1
		return isSourceA - isSourceB
	})
})

const visibleTags = computed(() => filteredTags.value?.slice(0, props.maxTags))
const overflowTags = computed(() => [
	...new Set([
		...(props.tags.filter((x) => !visibleTags.value?.includes(x)) ?? []),
		...(sortedExtraTags.value?.filter((x) => !visibleTags.value?.includes(x)) ?? []),
	]),
])
</script>

<template>
	<TagTagItem
		v-for="tag in visibleTags"
		:key="'visible-tag-' + tag"
		hide-non-loader-icon
		:tag="tag"
	/>
	<TagsOverflow
		v-if="overflowTags"
		:tags="overflowTags"
		class="smart-clickable:allow-pointer-events"
	/>
</template>
