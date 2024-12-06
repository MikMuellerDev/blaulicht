<svelte:options runes={false} />

<script lang="ts">
    import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import Select, { Option } from '@smui/select';
    import Button from '@smui/button';
    import { listen } from '@tauri-apps/api/event';
    import Bulb from "../components/Bulb.svelte";

  interface Device {
    host: string,
    device: string,
  }

  type Heartbeat = {
    seq: number;
  }

  async function selectDevice(device: Device) {
    console.log("SELECT DEVICE", device)
    const res = await invoke("select_device", device as any);
    console.log(res)
  }

  async function listDevices(): Promise<Device[]> {
    const devices: Device[] = await invoke("list_devices");
    console.log(devices)
    return devices
  }

  let devices: Device[]= [];
  let selected_device: Device;

  let visualizeCanvasGraph: HTMLCanvasElement | null = null;
  let visualizeCanvasGraphContext2D: CanvasRenderingContext2D | null | undefined = null

  let visualizeCanvas: HTMLCanvasElement | null = null;
  let visualizeCanvasContext2D: CanvasRenderingContext2D | null | undefined = null

  let volumePercent = 0;
  let lastVolumePercent = 0;

  let heartBeatActiveToggle = false
  function onHeartbeat() {
    heartBeatActiveToggle = !heartBeatActiveToggle
  }

  let speed = 0

  function msgHandler(payload: any) {
        // TODO: Check if this is actually volume?
        if (payload.Volume) {
            volumePercent = payload.Volume
        } else if (payload === "Heartbeat") {
            onHeartbeat()
        } else if (payload.Beat !== null && payload.Beat !== undefined) {
            beatSignal = payload.Beat === 1 ? true : false
        } else if (payload.Speed) {
            speed = payload.Speed
        }
  }

  function transition(oldV: number, newV: number): number[] {
    if (oldV > newV) {
        let array = []
        for (let i = oldV; i >= newV; i--) {
            array.push(i)
        }
        return array
    } else if (newV > oldV) {
        let array = []
        for (let i = oldV; i <= newV; i++) {
            array.push(i)
        }
        return array
    } else {
        return [oldV]
    }
  }

  const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

  let logs: string[] = []

const visualizeCanvasHeight = 200
const visualizeCanvasWidth = 50

  // async function drawVolume2() {
  //   let transitionPoints = transition(lastVolumePercent, volumePercent)
  //   for (let step of transitionPoints) {
  //       visualizeCanvasContext2D?.clearRect(0, 0, visualizeCanvasWidth, visualizeCanvasHeight)
  //       visualizeCanvasContext2D?.fillRect(
  //           0,
  //           visualizeCanvasHeight * ((100 - step) / 100),
  //           visualizeCanvasWidth,
  //           visualizeCanvasHeight - (visualizeCanvasHeight * ((100 - step) / 100)),
  //       )
  //
  //       visualizeCanvasContext2D!.font = "14px monospace";
  //       visualizeCanvasContext2D!.fillStyle = "black";
  //       visualizeCanvasContext2D?.fillText(`${step}`, 4, 14);
  //       visualizeCanvasContext2D!.fillStyle = "blue";
  //       await sleep(1)
  //   }
  //
  //   lastVolumePercent = volumePercent
  //   volumePercent-= 0.1
  // }

  async function drawVolume() {
        const colorMap = ['green', 'yellow', 'red']

        visualizeCanvasContext2D?.clearRect(0, 0, visualizeCanvasWidth, visualizeCanvasHeight)
        visualizeCanvasContext2D?.fillRect(
            0,
            visualizeCanvasHeight * ((100 - volumePercent) / 100),
            visualizeCanvasWidth,
            visualizeCanvasHeight - (visualizeCanvasHeight * ((100 - volumePercent) / 100)),
        )

        visualizeCanvasContext2D!.font = "14px monospace";
        visualizeCanvasContext2D!.fillStyle = 'black'
        visualizeCanvasContext2D?.fillText(`${volumePercent}`, 4, 22);

        const colorMapIndex = Math.floor(volumePercent / 100 * colorMap.length)
        visualizeCanvasContext2D!.fillStyle = colorMap[colorMapIndex];
        visualizeCanvasContext2D?.fillRect(
            0,
            0,
            50,
            5
        )

        visualizeCanvasContext2D!.fillStyle = 'white'
  }

  async function draw() {
        drawVolume()
        window.requestAnimationFrame(draw)
  }

  function initializeVolumeCanvas() {
    visualizeCanvasContext2D = visualizeCanvas?.getContext("2d")
    if (!visualizeCanvas || !visualizeCanvasContext2D) {
        console.error("Broken")
        return
    }

    visualizeCanvas!.style.width = `${visualizeCanvasWidth}`
    visualizeCanvas!.style.height = `${visualizeCanvasHeight}`
    visualizeCanvas!.width = visualizeCanvasWidth
    visualizeCanvas!.height = visualizeCanvasHeight
    visualizeCanvasContext2D!.fillStyle = 'white'
  }

  // function initializeGraphCanvas() {
  //   visualizeCanvasGraphContext2D = visualizeCanvasGraph?.getContext("2d")
  //   if (!visualizeCanvasGraph || !visualizeCanvasGraphContext2D) {
  //       console.error("Broken")
  //       return
  //   }
  //
  //   const visualizeCanvasHeight = 50
  //   const visualizeCanvasWidth = 300
  //
  //   visualizeCanvasGraph!.style.width = `${visualizeCanvasWidth}`
  //   visualizeCanvasGraph!.style.height = `${visualizeCanvasHeight}`
  //   visualizeCanvasGraph!.width = visualizeCanvasWidth
  //   visualizeCanvasGraph!.height = visualizeCanvasHeight
  //   visualizeCanvasGraphContext2D!.fillStyle = 'white'
  // }

  function intializeCanvases() {
    initializeVolumeCanvas()
    // initializeGraphCanvas()
    window.requestAnimationFrame(draw);
  }

  onMount(async () => {
        logs = [ 'Initializing...' ]

        listen<Heartbeat>('msg', (event) => {
            // console.log(`Received Message`, event.payload);
            msgHandler(event.payload)
        });

        const sock = await invoke("socket")
        logs = [...logs, `Socket initialized: ${sock}`]

        intializeCanvases()

        devices = await listDevices()
        selected_device = devices[0]

        logs = [...logs, "Ready."]
  })

  let beatSignal = false
</script>

<main class="container">
    <div class='top_bar'>
        <div class="top_bar__element">
            <Bulb size={20} bind:active={heartBeatActiveToggle}></Bulb>
            <span>Heartbeat</span>
        </div>

        <div class="top_bar__element">
            <code class='top_bar__speed__amount' class:bad={speed > 1000}>{speed}</code>us
            <span>Speed</span>
        </div>
    </div>

    <div class="main">
        <div class="main__left">
            <div class='below-center'>
                Volume
                <br>
                <canvas id="visualize_canvas" bind:this={visualizeCanvas}></canvas>
            </div>

            <div class='below-center'>
                Beat
                <Bulb size={40} bind:active={beatSignal}></Bulb>
            </div>

            <!-- <code> -->
            <!--     {#each logs as log} -->
            <!--         {log} -->
            <!--         <br> -->
            <!--     {/each} -->
            <!-- </code> -->
        </div>
        <div class="main__right">
            <div>
                <!-- <div> -->
                <!--     <canvas id="visualize_canvas_graph" bind:this={visualizeCanvasGraph}></canvas> -->
                <!-- </div> -->

                <Select bind:value={selected_device} label="Select Input Device">
                {#each devices as dev}
                    <Option value={dev}>{dev.host} | {dev.device}</Option>
                {/each}
                </Select>

                <Button onclick={() => selectDevice(selected_device)}>Select</Button>

                <pre class="status">
                    Selected:
                    {#if selected_device}
                        {selected_device.host} | {selected_device.device}
                    {:else}
                        None
                    {/if}
                </pre>
            </div>
    </div>
</main>

    <!-- <div class="columns margins" style="justify-content: flex-start;"> -->
    <!--     <div class="split"> -->
    <!--         <canvas id="visualize_canvas" bind:this={visualizeCanvas}></canvas> -->
    <!---->
    <!--     <code> -->
    <!--             {#each logs as log} -->
    <!--                 {log} -->
    <!--                 <br> -->
    <!--             {/each} -->
    <!--         </code> -->
    <!--     </div> -->
    <!---->
    <!-- <div> -->
    <!--     <Select bind:value={selected_device} label="Select Input Device"> -->
    <!--     {#each devices as dev} -->
    <!--         <Option value={dev}>{dev.host} | {dev.dev}</Option> -->
    <!--     {/each} -->
    <!--     </Select> -->
    <!---->
    <!--     <Button on:click={() => selectDevice(selected_device)}>Select</Button> -->
    <!---->
    <!--     <pre class="status"> -->
    <!--         Selected: -->
    <!--         {#if selected_device} -->
    <!--             {selected_device.host} | {selected_device.dev} -->
    <!--         {:else} -->
    <!--             None -->
    <!--         {/if} -->
    <!--     </pre> -->
    <!-- </div> -->
    <!-- </div> -->
    <!---->
    <!-- <ul> -->
    <!--     {#each devices as device } -->
    <!--         <li> -->
    <!--             {JSON.stringify(device)} -->
    <!--         </li> -->
    <!--     {/each} -->
    <!-- </ul> -->

<style>
    :root {
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 24px;
        font-weight: 400;

        font-synthesis: none;
        text-rendering: optimizeLegibility;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
        -webkit-text-size-adjust: 100%;

        background-color: #0f0f0f;
        color: #f6f6f6;
    }

    .below-center {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
    }

    .container {
        margin: 0;
        height: 100%;
        box-sizing: border-box;
    }

    .top_bar {
        background-color: rgba(255, 255, 255, 0.3);
        display: flex;
        gap: 3rem;
        padding: 0.5rem 1rem;
    }

    .top_bar__element {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .top_bar__speed__amount {
        width: 2rem;

        &.bad {
            color: red;
        }
    }

    .main {
        display: flex;
        width: 100%;
        height: 100%;
    }

    .main__left {
        background-color: rgba(255, 255, 255, 0.2);
        width: 10%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        /* justify-content: space-between; */
        padding: 0.5rem 0.5rem;
    }

    .main__right {
        background-color: rgba(255, 255, 255, 0.1);
        width: 90%;
        height: 100%;
    }

    #visualize_canvas {
        background-color: gray;
    }

    #visualize_canvas_graph {
        background-color: gray;
    }
</style>
