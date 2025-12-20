<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/store.svelte';
  import TopBar from './components/TopBar.svelte';
  import NavBar from './components/NavBar.svelte';
  import Toast from './components/Toast.svelte';
  import StatusTab from './routes/StatusTab.svelte';
  import ConfigTab from './routes/ConfigTab.svelte';
  import ModulesTab from './routes/ModulesTab.svelte';
  import LogsTab from './routes/LogsTab.svelte';
  import InfoTab from './routes/InfoTab.svelte';
  import GranaryTab from './routes/GranaryTab.svelte';
  import WinnowingTab from './routes/WinnowingTab.svelte';
  import './app.css';
  import './layout.css';
  
  const TABS = ['status', 'config', 'modules', 'logs', 'granary', 'winnowing', 'info'];
  
  let activeTab = $state('status');
  let dragOffset = $state(0);
  let isDragging = $state(false);
  let containerWidth = $state(0);
  let touchStartX = 0;
  let touchStartY = 0;
  let isReady = $state(false);

  function switchTab(id: string) {
    activeTab = id;
  }
  
  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.changedTouches[0].screenX;
    touchStartY = e.changedTouches[0].screenY;
    isDragging = true;
    dragOffset = 0;
  }

  function handleTouchMove(e: TouchEvent) {
    if (!isDragging) return;
    const currentX = e.changedTouches[0].screenX;
    const currentY = e.changedTouches[0].screenY;
    let diffX = currentX - touchStartX;
    const diffY = currentY - touchStartY;
    
    // Lock vertical scroll
    if (Math.abs(diffY) > Math.abs(diffX)) return;
    
    if (e.cancelable) e.preventDefault();

    const currentIndex = TABS.indexOf(activeTab);
    // Resistance at edges
    if ((currentIndex === 0 && diffX > 0) || (currentIndex === TABS.length - 1 && diffX < 0)) {
      diffX = diffX / 3;
    }
    dragOffset = diffX;
  }

  function handleTouchEnd() {
    if (!isDragging) return;
    isDragging = false;
    const threshold = containerWidth * 0.33 || 80;
    const currentIndex = TABS.indexOf(activeTab);
    let nextIndex = currentIndex;

    if (dragOffset < -threshold && currentIndex < TABS.length - 1) {
      nextIndex = currentIndex + 1;
    } else if (dragOffset > threshold && currentIndex > 0) {
      nextIndex = currentIndex - 1;
    }

    if (nextIndex !== currentIndex) {
      switchTab(TABS[nextIndex]);
    }
    dragOffset = 0;
  }

  onMount(async () => {
    try {
      await store.init();
    } finally {
      isReady = true;
    }
  });

  let baseTranslateX = $derived(TABS.indexOf(activeTab) * -(100 / TABS.length));
</script>

<div class="app-root">
  {#if !isReady}
    <div class="loading-container">
       <div class="spinner"></div>
       <span class="loading-text">Loading...</span>
    </div>
  {:else}
    <TopBar />
    <main class="main-content" 
          bind:clientWidth={containerWidth}
          ontouchstart={handleTouchStart} 
          ontouchmove={handleTouchMove}
          ontouchend={handleTouchEnd}
          ontouchcancel={handleTouchEnd}>
      <div class="swipe-track"
           style:transform={`translateX(calc(${baseTranslateX}% + ${dragOffset}px))`}
           style:width={`${TABS.length * 100}%`}
           style:transition={isDragging ? 'none' : 'transform 0.3s cubic-bezier(0.25, 0.8, 0.5, 1)'}>
        
        <div class="swipe-page" style:width={`${100 / TABS.length}%`}><div class="page-scroller"><StatusTab /></div></div>
        <div class="swipe-page" style:width={`${100 / TABS.length}%`}><div class="page-scroller"><ConfigTab /></div></div>
        <div class="swipe-page" style:width={`${100 / TABS.length}%`}><div class="page-scroller"><ModulesTab /></div></div>
        <div class="swipe-page" style:width={`${100 / TABS.length}%`}><div class="page-scroller"><LogsTab /></div></div>
        <div class="swipe-page" style:width={`${100 / TABS.length}%`}><div class="page-scroller"><GranaryTab /></div></div>
        <div class="swipe-page" style:width={`${100 / TABS.length}%`}><div class="page-scroller"><WinnowingTab /></div></div>
        <div class="swipe-page" style:width={`${100 / TABS.length}%`}><div class="page-scroller"><InfoTab /></div></div>
        
      </div>
    </main>
    <NavBar {activeTab} onTabChange={switchTab} />
  {/if}
  <Toast />
</div>

<style>
  .loading-container {
    display: flex; 
    justify-content: center; 
    align-items: center; 
    height: 100vh; 
    flex-direction: column; 
    gap: 16px;
  }
  .loading-text {
    opacity: 0.6;
    font-family: var(--md-ref-typeface-plain);
  }
  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--md-sys-color-surface-container-high, #e0e0e0);
    border-top-color: var(--md-sys-color-primary, #6750a4);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>