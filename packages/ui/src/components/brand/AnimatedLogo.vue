<template>
	<div class="flex items-center justify-center p-2">
		<svg
			class="animate-spin text-brand"
			width="48"
			height="48"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2.5"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<circle cx="12" cy="12" r="10" stroke-opacity="0.2" />
			<path d="M12 2a10 10 0 0 1 10 10" />
		</svg>
	</div>
</template>

<script setup>
</script>

<style scoped>
.animate-spin {
	animation: spin 1s linear infinite;
}
@keyframes spin {
	from {
		transform: rotate(0deg);
	}
	to {
		transform: rotate(360deg);
	}
}
</style>
