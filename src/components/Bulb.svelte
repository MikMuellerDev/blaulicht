<svelte:options runes={false} />

<script lang="ts">
    import { onMount } from "svelte";

    export let active = false
    export let size = 30

    export let activeColor = "lime"
    export let passiveColor = "black"

    let visualizeCanvas: HTMLCanvasElement | null = null;
    let visualizeCanvasContext2D: CanvasRenderingContext2D | null | undefined = null

    async function draw() {
        visualizeCanvasContext2D!.fillStyle = active ? activeColor : passiveColor;
        visualizeCanvasContext2D?.arc(
            size / 2,
            size / 2,
            size / 2,
            0,
            2 * Math.PI,
        );
        visualizeCanvasContext2D?.fill()
        window.requestAnimationFrame(draw)
    }

    function initializeCanvas() {
        visualizeCanvas!.style.width = `${size}`
        visualizeCanvas!.style.height = `${size}`
        visualizeCanvas!.width = size
        visualizeCanvas!.height = size
        visualizeCanvasContext2D!.fillStyle = 'white'
        window.requestAnimationFrame(draw);
    }

    onMount(async () => {
            visualizeCanvasContext2D = visualizeCanvas?.getContext("2d")
            if (!visualizeCanvas || !visualizeCanvasContext2D) {
                console.error("Broken")
                return
            }

            initializeCanvas()
    })
</script>

<canvas class="bulb" bind:this={visualizeCanvas}></canvas>

<style>
    .bulb {
        background-color: transparent;
    }
</style>
