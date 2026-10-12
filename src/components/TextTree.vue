<template>
	<ul class="tree" role="tree" :aria-label="label">
		<li v-for="node in localNodes" :key="node.id" role="treeitem" :aria-expanded="node.children?.length ? node.expanded !== false : undefined">
			<div
				class="tree-row"
				:class="{ dragging: draggedId === node.id, 'drop-before': dropTarget?.id === node.id && dropTarget.position === 'before', 'drop-after': dropTarget?.id === node.id && dropTarget.position === 'after', 'drop-inside': dropTarget?.id === node.id && dropTarget.position === 'inside' }"
				draggable="true"
				tabindex="0"
				@dragstart="startDrag(node, $event)"
				@dragend="endDrag"
				@dragover.prevent="updateDropTarget(node, $event)"
				@dragleave="clearDropTarget(node)"
				@drop.prevent="drop(node, $event)"
				@keydown.enter="toggle(node)"
				@keydown.space.prevent="toggle(node)"
			>
				<button
					v-if="node.children?.length"
					class="toggle"
					type="button"
					:aria-label="node.expanded === false ? `Expand ${node.label}` : `Collapse ${node.label}`"
					@click.stop="toggle(node)"
				>
					{{ node.expanded === false ? '+' : '-' }}
				</button>
				<span v-else class="toggle-spacer" aria-hidden="true"></span>
				<span class="node-label">{{ node.label }}</span>
			</div>

			<TextTree
				v-if="node.children?.length && node.expanded !== false"
				:nodes="node.children"
				:label="`${node.label} children`"
				:is-root="false"
				@move="handleMove"
			/>
		</li>
	</ul>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";

export interface TreeNode {
	id: string;
	label: string;
	children?: TreeNode[];
	expanded?: boolean;
}

type DropPosition = "before" | "inside" | "after";
interface MoveEvent {
	id: string;
	targetId: string;
	position: DropPosition;
}

const props = withDefaults(defineProps<{
	nodes: TreeNode[];
	label?: string;
	isRoot?: boolean;
}>(), {
	label: "Text tree",
	isRoot: true,
});

const emit = defineEmits<{
	(event: "update:nodes", nodes: TreeNode[]): void;
	(event: "move", move: MoveEvent): void;
}>();

const localNodes = ref<TreeNode[]>(cloneNodes(props.nodes));
const draggedId = ref<string | null>(null);
const dropTarget = ref<{ id: string; position: DropPosition } | null>(null);

watch(() => props.nodes, (nodes) => {
	localNodes.value = cloneNodes(nodes);
}, { deep: true });

function cloneNodes(nodes: TreeNode[]): TreeNode[] {
	return nodes.map((node) => ({
		...node,
		children: node.children ? cloneNodes(node.children) : undefined,
	}));
}

function startDrag(node: TreeNode, event: DragEvent) {
	draggedId.value = node.id;
	event.dataTransfer?.setData("text/plain", node.id);
	if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}

function endDrag() {
	draggedId.value = null;
	dropTarget.value = null;
}

function updateDropTarget(node: TreeNode, event: DragEvent) {
	const sourceId = draggedId.value;
	if (!sourceId || sourceId === node.id || isDescendant(localNodes.value, sourceId, node.id)) {
		dropTarget.value = null;
		return;
	}

	const row = event.currentTarget as HTMLElement;
	const ratio = (event.clientY - row.getBoundingClientRect().top) / row.offsetHeight;
	const position: DropPosition = ratio < 0.25 ? "before" : ratio > 0.75 ? "after" : "inside";
	dropTarget.value = { id: node.id, position };
}

function clearDropTarget(node: TreeNode) {
	if (dropTarget.value?.id === node.id) dropTarget.value = null;
}

function drop(node: TreeNode, event: DragEvent) {
	const sourceId = event.dataTransfer?.getData("text/plain") || draggedId.value;
	const position = dropTarget.value?.id === node.id ? dropTarget.value.position : "inside";
	if (!sourceId || sourceId === node.id) return endDrag();

	const move = { id: sourceId, targetId: node.id, position } satisfies MoveEvent;
	if (props.isRoot) applyMove(move);
	else emit("move", move);
	endDrag();
}

function handleMove(move: MoveEvent) {
	if (props.isRoot) applyMove(move);
	else emit("move", move);
}

function applyMove(move: MoveEvent) {
	const source = removeNode(localNodes.value, move.id);
	if (!source || isDescendant([source], move.targetId, move.id)) return;
	insertNode(localNodes.value, source, move.targetId, move.position);
	emit("update:nodes", cloneNodes(localNodes.value));
}

function removeNode(nodes: TreeNode[], id: string): TreeNode | null {
	const index = nodes.findIndex((node) => node.id === id);
	if (index !== -1) return nodes.splice(index, 1)[0];
	for (const node of nodes) {
		const removed = node.children && removeNode(node.children, id);
		if (removed) return removed;
	}
	return null;
}

function insertNode(nodes: TreeNode[], source: TreeNode, targetId: string, position: DropPosition): boolean {
	const targetIndex = nodes.findIndex((node) => node.id === targetId);
	if (targetIndex !== -1) {
		if (position === "inside") {
			const target = nodes[targetIndex];
			target.children ||= [];
			target.children.push(source);
			target.expanded = true;
		} else {
			nodes.splice(position === "before" ? targetIndex : targetIndex + 1, 0, source);
		}
		return true;
	}
	return nodes.some((node) => node.children && insertNode(node.children, source, targetId, position));
}

function isDescendant(nodes: TreeNode[], sourceId: string, possibleAncestorId: string): boolean {
	const ancestor = findNode(nodes, possibleAncestorId);
	return Boolean(ancestor?.children && findNode(ancestor.children, sourceId));
}

function findNode(nodes: TreeNode[], id: string): TreeNode | null {
	for (const node of nodes) {
		if (node.id === id) return node;
		const found = node.children && findNode(node.children, id);
		if (found) return found;
	}
	return null;
}

function toggle(node: TreeNode) {
	if (node.children?.length) node.expanded = node.expanded === false;
}
</script>

<style scoped>
.tree {
	margin: 0;
	padding-left: 1.25rem;
	list-style: none;
}

.tree-row {
	position: relative;
	display: flex;
	align-items: center;
	min-height: 2.5rem;
	gap: 0.5rem;
	padding: 0.35rem 0.6rem;
	border: 1px solid transparent;
	border-radius: 0.35rem;
	cursor: grab;
	user-select: none;
}

.tree-row:focus-visible,
.tree-row:hover {
	background: #eef4f2;
	outline: none;
}

.tree-row.dragging {
	opacity: 0.45;
}

.tree-row.drop-before::before,
.tree-row.drop-after::after {
	position: absolute;
	right: 0;
	left: 0;
	height: 2px;
	background: #13795b;
	content: "";
}

.tree-row.drop-before::before { top: -2px; }
.tree-row.drop-after::after { bottom: -2px; }
.tree-row.drop-inside { border-color: #13795b; background: #e1f2eb; }

.toggle,
.toggle-spacer {
	display: grid;
	width: 1.25rem;
	height: 1.25rem;
	flex: 0 0 1.25rem;
	place-items: center;
}

.toggle {
	padding: 0;
	border: 0;
	border-radius: 0.2rem;
	color: #13795b;
	background: transparent;
	font-weight: 700;
	cursor: pointer;
}

.toggle:hover { background: #cbe7dc; }
.node-label { overflow-wrap: anywhere; }
</style>