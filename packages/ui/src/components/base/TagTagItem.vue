<template>
	<TagItem :action="action" :style="customStyle">
		<component :is="icon" v-if="icon" />
		<FormattedTag :tag="tag" />
	</TagItem>
</template>
<script setup lang="ts">
import { getTagIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { getTagMessage } from '../../utils'
import FormattedTag from './FormattedTag.vue'
import TagItem from './TagItem.vue'

const props = withDefaults(
	defineProps<{
		tag: string
		hideNonLoaderIcon?: boolean
		action?: (event: MouseEvent) => void
	}>(),
	{
		hideNonLoaderIcon: false,
		action: undefined,
	},
)

const icon = computed(() =>
	props.hideNonLoaderIcon && !isLoader.value ? undefined : getTagIcon(props.tag),
)
const isLoader = computed(() => getTagMessage(props.tag, 'loader') !== undefined)

const customStyle = computed(() => {
	const lower = props.tag.toLowerCase()
	if (lower === 'curseforge') {
		return '--_bg-color: rgba(241, 100, 54, 0.25); --_color: #ff6a38; font-weight: 700;'
	}
	if (lower === 'modrinth') {
		return '--_bg-color: rgba(27, 217, 106, 0.25); --_color: #1bd96a; font-weight: 700;'
	}
	return isLoader.value ? `--_color: var(--color-platform-${props.tag})` : ''
})
</script>
