<script setup lang="ts">
import { CheckIcon, CopyIcon, TriangleAlertIcon, WrenchIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { ref } from 'vue'
import { useRouter } from 'vue-router'

import { edit } from '@/helpers/instance'

export interface CrashDiagnosticReport {
	has_crashed: bool
	error_type: string
	summary: string
	recommendation: string
	target_mod_id?: string | null
	target_mod_name?: string | null
	required_version?: string | null
	recommended_ram_mb?: number | null
	recommended_java_version?: number | null
	log_snippet?: string | null
}

const props = defineProps<{
	instanceId: string
	report: CrashDiagnosticReport
}>()

const emit = defineEmits(['dismiss', 'repaired'])

const router = useRouter()
const showSnippet = ref(false)
const copied = ref(false)
const fixing = ref(false)

async function copySnippet() {
	if (!props.report.log_snippet) return
	try {
		await navigator.clipboard.writeText(props.report.log_snippet)
		copied.value = true
		setTimeout(() => {
			copied.value = false
		}, 2000)
	} catch (e) {
		console.error('Failed to copy snippet', e)
	}
}

async function handleAutoFix() {
	fixing.value = true
	try {
		if (props.report.error_type === 'out_of_memory' && props.report.recommended_ram_mb) {
			await edit(props.instanceId, {
				max_memory: props.report.recommended_ram_mb,
			})
			emit('repaired')
		} else if (props.report.error_type === 'missing_dependency' && props.report.target_mod_id) {
			router.push(`/browse/mod?query=${encodeURIComponent(props.report.target_mod_id)}`)
		} else if (props.report.error_type === 'java_mismatch') {
			router.push(`/instance/${props.instanceId}/options`)
		} else {
			router.push(`/instance/${props.instanceId}/logs`)
		}
	} catch (e) {
		console.error('Auto fix error', e)
	} finally {
		fixing.value = false
	}
}
</script>

<template>
	<div
		v-if="report && report.has_crashed"
		class="relative mb-6 rounded-xl border border-red-500/40 bg-red-950/30 p-4 shadow-lg backdrop-blur-md transition-all"
	>
		<div class="flex items-start justify-between gap-3">
			<div class="flex items-start gap-3">
				<div class="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-red-500/20 text-red-400">
					<TriangleAlertIcon class="h-5 w-5" />
				</div>
				<div>
					<h4 class="font-bold text-red-200 text-base flex items-center gap-2">
						Local Crash Diagnostic
						<span class="rounded bg-red-500/20 px-2 py-0.5 text-xs font-medium text-red-300 capitalize">
							{{ report.error_type.replace('_', ' ') }}
						</span>
					</h4>
					<p class="mt-1 text-sm font-medium text-gray-200">
						{{ report.summary }}
					</p>
					<p class="mt-1 text-xs text-gray-400">
						💡 <span class="font-semibold text-gray-300">Recommendation:</span> {{ report.recommendation }}
					</p>
				</div>
			</div>

			<button
				class="text-gray-400 hover:text-gray-200 p-1 rounded-md transition-colors"
				title="Dismiss crash notification"
				@click="emit('dismiss')"
			>
				<XIcon class="h-4 w-4" />
			</button>
		</div>

		<!-- Action Bar -->
		<div class="mt-4 flex flex-wrap items-center gap-2 border-t border-red-500/20 pt-3">
			<button
				v-if="report.error_type === 'out_of_memory'"
				class="flex items-center gap-1.5 rounded-lg bg-emerald-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-emerald-500 transition-colors shadow-sm"
				:disabled="fixing"
				@click="handleAutoFix"
			>
				<WrenchIcon class="h-3.5 w-3.5" />
				1-Click Fix: Increase RAM to 6GB
			</button>

			<button
				v-else-if="report.error_type === 'missing_dependency'"
				class="flex items-center gap-1.5 rounded-lg bg-brand px-3 py-1.5 text-xs font-semibold text-white hover:bg-brand-hover transition-colors shadow-sm"
				@click="handleAutoFix"
			>
				<WrenchIcon class="h-3.5 w-3.5" />
				Find & Install Missing Dependency
			</button>

			<button
				v-else-if="report.error_type === 'java_mismatch'"
				class="flex items-center gap-1.5 rounded-lg bg-amber-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-amber-500 transition-colors shadow-sm"
				@click="handleAutoFix"
			>
				<WrenchIcon class="h-3.5 w-3.5" />
				Open Java Settings
			</button>

			<button
				v-if="report.log_snippet"
				class="flex items-center gap-1.5 rounded-lg border border-gray-700 bg-gray-800/80 px-3 py-1.5 text-xs font-medium text-gray-300 hover:bg-gray-700 transition-colors"
				@click="showSnippet = !showSnippet"
			>
				{{ showSnippet ? 'Hide Log Snippet' : 'View Log Snippet' }}
			</button>

			<button
				v-if="report.log_snippet"
				class="flex items-center gap-1.5 rounded-lg border border-gray-700 bg-gray-800/80 px-3 py-1.5 text-xs font-medium text-gray-300 hover:bg-gray-700 transition-colors"
				@click="copySnippet"
			>
				<CheckIcon v-if="copied" class="h-3.5 w-3.5 text-emerald-400" />
				<CopyIcon v-else class="h-3.5 w-3.5" />
				{{ copied ? 'Copied!' : 'Copy Snippet' }}
			</button>
		</div>

		<!-- Log Snippet Drawer -->
		<div
			v-if="showSnippet && report.log_snippet"
			class="mt-3 rounded-lg border border-gray-800 bg-gray-950 p-3 text-xs font-mono text-gray-300 overflow-x-auto max-h-48 scrollbar-thin"
		>
			<pre class="whitespace-pre-wrap leading-relaxed">{{ report.log_snippet }}</pre>
		</div>
	</div>
</template>
