// osm-pathfinder — Frontend Application Controller

document.addEventListener('DOMContentLoaded', () => {
  // -------------------------------------------------------------------------
  // State
  // -------------------------------------------------------------------------
  let currentMode = 'routing';
  let startLatLng = null;
  let endLatLng = null;
  let startMarker = null;
  let endMarker = null;
  let routePolyline = null;
  let selectedAlgorithm = 'astar';
  let selectedMetric = 'distance';
  let selectedDepartureTime = '08:15';

  let isoCenterLatLng = [11.5564, 104.9282];
  let isoCenterMarker = null;
  let isoBuckets = [10, 20, 30, 45];

  const CITY_HUBS = {
    'pp': [11.5564, 104.9282],
    'siemreap': [13.3671, 103.8448],
    'battambang': [13.0957, 103.2022],
    'sihanoukville': [10.6253, 103.5234],
    'kampot': [10.6104, 104.1815]
  };

  // -------------------------------------------------------------------------
  // Delivery & Highway Presets
  // -------------------------------------------------------------------------
  const PRESETS = {
    // --- Phnom Penh Urban Delivery Corridors ---
    'pp-depot-a-to-st271': {
      start: [11.5680, 104.9223],
      end: [11.5305, 104.9085],
      name: 'Central Market Depot A → St 271 (Meanchey)',
      category: 'urban',
      desc: 'Central Market to Boeung Tumpun • ~6.8 km',
      tag: 'Depot A'
    },
    'pp-depot-a-to-bkk1': {
      start: [11.5680, 104.9223],
      end: [11.5528, 104.9282],
      name: 'Central Market Depot A → BKK1 (Pasteur)',
      category: 'urban',
      desc: 'Depot A to Pasteur (St 51) • ~2.1 km',
      tag: 'Depot A'
    },
    'pp-depot-b-to-tuolkork': {
      start: [11.5435, 104.9142],
      end: [11.5732, 104.8984],
      name: 'Russian Market Depot B → Tuol Kork (St 289)',
      category: 'urban',
      desc: 'Toul Tompoung to TK Avenue (St 289) • ~5.2 km',
      tag: 'Depot B'
    },
    'pp-hub-to-riverside': {
      start: [11.5621, 104.9160],
      end: [11.5695, 104.9312],
      name: 'Central Hub (Bak Touk) → Riverside (Sisowath)',
      category: 'urban',
      desc: 'Olympic to Sisowath Quay waterfront • ~2.4 km',
      tag: 'Central Hub'
    },
    'pp-depot-a-to-sensok': {
      start: [11.5680, 104.9223],
      end: [11.5850, 104.8820],
      name: 'Central Market Depot A → Sen Sok (AEON 2)',
      category: 'urban',
      desc: 'Depot A to AEON Mall 2 via Russian Blvd • ~6.4 km',
      tag: 'Depot A'
    },
    'pp-depot-b-to-norodom': {
      start: [11.5435, 104.9142],
      end: [11.5564, 104.9282],
      name: 'Russian Market Depot B → Independence Monument',
      category: 'urban',
      desc: 'Toul Tompoung north to Norodom Blvd • ~2.3 km',
      tag: 'Depot B'
    },
    // --- Cambodia Highway Corridors ---
    'pp-siemreap': {
      start: [11.5564, 104.9282],
      end: [13.3671, 103.8448],
      name: 'Phnom Penh → Siem Reap (NR6)',
      category: 'highway',
      desc: 'National Road 6 Corridor • ~265 km',
      tag: 'NR6'
    },
    'pp-sihanoukville': {
      start: [11.5564, 104.9282],
      end: [10.6253, 103.5234],
      name: 'Phnom Penh → Sihanoukville (Expressway)',
      category: 'highway',
      desc: 'Phnom Penh–Sihanoukville Expressway • ~190 km',
      tag: 'Expressway'
    },
    'pp-battambang': {
      start: [11.5564, 104.9282],
      end: [13.0957, 103.2022],
      name: 'Phnom Penh → Battambang (NR5)',
      category: 'highway',
      desc: 'National Road 5 Northwest Corridor • ~290 km',
      tag: 'NR5'
    },
    'siemreap-battambang': {
      start: [13.3671, 103.8448],
      end: [13.0957, 103.2022],
      name: 'Siem Reap → Battambang (NR6/NR5)',
      category: 'highway',
      desc: 'NR6 West via Kralanh to NR5 • ~165 km',
      tag: 'NR6/NR5'
    },
    'pp-kampot': {
      start: [11.5564, 104.9282],
      end: [10.6104, 104.1815],
      name: 'Phnom Penh → Kampot (NR3)',
      category: 'highway',
      desc: 'National Road 3 Southern Corridor • ~148 km',
      tag: 'NR3'
    },
    'sr-angkor': {
      start: [13.3671, 103.8448],
      end: [13.4125, 103.8670],
      name: 'Siem Reap Central → Angkor Wat',
      category: 'highway',
      desc: 'Old Market to Angkor Wat temple • ~6.5 km',
      tag: 'Angkor'
    }
  };

  // -------------------------------------------------------------------------
  // Leaflet Map Initialization
  // -------------------------------------------------------------------------
  const map = L.map('map', {
    zoomControl: false
  }).setView([12.5657, 104.9910], 7); // Center of Cambodia

  L.control.zoom({ position: 'topright' }).addTo(map);

  // Modern OpenStreetMap tile layer
  L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '© OpenStreetMap contributors'
  }).addTo(map);

  // Layer group for rendering algorithm search wavefronts
  const wavefrontLayer = L.layerGroup().addTo(map);
  // Layer group for rendering isochrone contours
  const isoLayerGroup = L.layerGroup().addTo(map);

  // Custom Pin Icons
  const createPinIcon = (colorClass) => {
    return L.divIcon({
      className: `custom-pin ${colorClass}`,
      iconSize: [20, 20],
      iconAnchor: [10, 10]
    });
  };

  // -------------------------------------------------------------------------
  // DOM Elements
  // -------------------------------------------------------------------------
  const controlPanel = document.getElementById('control-panel');
  const sheetHandleBar = document.getElementById('sheet-handle-bar');
  const panelToggleBtn = document.getElementById('panel-toggle-btn');
  const panelHeader = document.getElementById('panel-header');
  const graphBadge = document.getElementById('graph-badge');
  const modeRoutingBtn = document.getElementById('mode-routing-btn');
  const modeIsochroneBtn = document.getElementById('mode-isochrone-btn');
  const routingPanel = document.getElementById('routing-panel');
  const isochronePanel = document.getElementById('isochrone-panel');
  const presetSelect = document.getElementById('preset-select');
  const startCoordsDisplay = document.getElementById('start-coords-display');
  const endCoordsDisplay = document.getElementById('end-coords-display');
  const calcRouteBtn = document.getElementById('calc-route-btn');
  const compareBtn = document.getElementById('compare-btn');
  const clearBtn = document.getElementById('clear-btn');
  const metricsCard = document.getElementById('metrics-card');
  const compareCard = document.getElementById('compare-card');
  const metricDistance = document.getElementById('metric-distance');
  const metricDuration = document.getElementById('metric-duration');
  const metricVisited = document.getElementById('metric-visited');
  const metricLatency = document.getElementById('metric-latency');
  const compareTbody = document.getElementById('compare-tbody');
  const compareInsight = document.getElementById('compare-insight');
  const algoCards = document.querySelectorAll('.algo-card');
  const metricDistBtn = document.getElementById('metric-dist-btn');
  const metricTimeBtn = document.getElementById('metric-time-btn');
  const trafficTimeSelect = document.getElementById('traffic-time');
  const chTrafficNote = document.getElementById('ch-traffic-note');
  const showWavefrontToggle = document.getElementById('show-wavefront-toggle');

  const isoPresetSelect = document.getElementById('iso-preset-select');
  const isoCoordsDisplay = document.getElementById('iso-coords-display');
  const calcIsoBtn = document.getElementById('calc-iso-btn');
  const clearIsoBtn = document.getElementById('clear-iso-btn');
  const isoResultsCard = document.getElementById('iso-results-card');
  const isoLegendList = document.getElementById('iso-legend-list');
  const chipBtns = document.querySelectorAll('.chip-btn');
  const instructionBanner = document.querySelector('.instruction-banner');

  // -------------------------------------------------------------------------
  // Mobile Panel Collapse / Expand Controls
  // -------------------------------------------------------------------------
  function togglePanel(collapseOnly = false) {
    if (!controlPanel) return;
    if (collapseOnly) {
      controlPanel.classList.add('collapsed');
    } else {
      controlPanel.classList.toggle('collapsed');
    }
  }

  function expandPanel() {
    if (controlPanel) controlPanel.classList.remove('collapsed');
  }

  if (panelToggleBtn) {
    panelToggleBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      togglePanel();
    });
  }

  if (sheetHandleBar) {
    sheetHandleBar.addEventListener('click', () => {
      togglePanel();
    });

    let touchStartY = 0;
    sheetHandleBar.addEventListener('touchstart', (e) => {
      touchStartY = e.touches[0].clientY;
    }, { passive: true });

    sheetHandleBar.addEventListener('touchend', (e) => {
      const touchEndY = e.changedTouches[0].clientY;
      const diffY = touchEndY - touchStartY;
      if (diffY > 30) {
        togglePanel(true); // Swipe down
      } else if (diffY < -30) {
        expandPanel();     // Swipe up
      }
    }, { passive: true });
  }

  if (panelHeader) {
    panelHeader.addEventListener('click', (e) => {
      if (e.target.closest('#panel-toggle-btn') || e.target.closest('select') || e.target.closest('button')) return;
      if (controlPanel && controlPanel.classList.contains('collapsed')) {
        expandPanel();
      }
    });
  }

  // -------------------------------------------------------------------------
  // Fetch Graph Stats on Startup
  // -------------------------------------------------------------------------
  async function fetchGraphStats() {
    try {
      const res = await fetch('/api/graph/stats');
      if (res.ok) {
        const data = await res.json();
        graphBadge.textContent = `${data.nodes.toLocaleString()} nodes | ${data.edges.toLocaleString()} edges`;
        graphBadge.classList.add('ready');
      } else {
        graphBadge.textContent = 'API connected';
      }
    } catch (e) {
      graphBadge.textContent = 'Demo Mode (Offline)';
    }
  }
  fetchGraphStats();

  // -------------------------------------------------------------------------
  // Optimization & Traffic Controls
  // -------------------------------------------------------------------------
  if (metricDistBtn && metricTimeBtn) {
    metricDistBtn.addEventListener('click', () => {
      metricDistBtn.classList.add('active');
      metricTimeBtn.classList.remove('active');
      selectedMetric = 'distance';
      if (startLatLng && endLatLng) calculateRoute();
    });

    metricTimeBtn.addEventListener('click', () => {
      metricTimeBtn.classList.add('active');
      metricDistBtn.classList.remove('active');
      selectedMetric = 'time';
      if (startLatLng && endLatLng) calculateRoute();
    });
  }

  if (trafficTimeSelect) {
    trafficTimeSelect.addEventListener('change', (e) => {
      selectedDepartureTime = e.target.value;
      if (startLatLng && endLatLng) calculateRoute();
    });
  }

  if (showWavefrontToggle) {
    showWavefrontToggle.addEventListener('change', () => {
      if (!showWavefrontToggle.checked) {
        wavefrontLayer.clearLayers();
      } else if (startLatLng && endLatLng) {
        calculateRoute();
      }
    });
  }

  updateTrafficControlState();

  // -------------------------------------------------------------------------
  // Mode Switcher (Routing vs. Isochrone)
  // -------------------------------------------------------------------------
  if (modeRoutingBtn && modeIsochroneBtn) {
    modeRoutingBtn.addEventListener('click', () => {
      currentMode = 'routing';
      modeRoutingBtn.classList.add('active');
      modeIsochroneBtn.classList.remove('active');
      routingPanel.classList.remove('hidden');
      isochronePanel.classList.add('hidden');
      if (instructionBanner) {
        instructionBanner.innerHTML = 'Click anywhere on the map to set <strong>Start</strong> (green) and <strong>Destination</strong> (red)';
      }
      isoLayerGroup.clearLayers();
      if (isoCenterMarker) {
        map.removeLayer(isoCenterMarker);
        isoCenterMarker = null;
      }
    });

    modeIsochroneBtn.addEventListener('click', () => {
      currentMode = 'isochrone';
      modeIsochroneBtn.classList.add('active');
      modeRoutingBtn.classList.remove('active');
      isochronePanel.classList.remove('hidden');
      routingPanel.classList.add('hidden');
      if (instructionBanner) {
        instructionBanner.innerHTML = 'Click anywhere on the map to set <strong>Reachability Origin</strong> (blue)';
      }
      if (routePolyline) map.removeLayer(routePolyline);
      if (startMarker) map.removeLayer(startMarker);
      if (endMarker) map.removeLayer(endMarker);
      wavefrontLayer.clearLayers();
      metricsCard.classList.add('hidden');
      compareCard.classList.add('hidden');

      setIsoCenter(isoCenterLatLng);
      calculateIsochrones();
    });
  }

  // -------------------------------------------------------------------------
  // Map Click Handler (Drop Pins)
  // -------------------------------------------------------------------------
  map.on('click', (e) => {
    const lat = parseFloat(e.latlng.lat.toFixed(5));
    const lon = parseFloat(e.latlng.lng.toFixed(5));

    if (currentMode === 'isochrone') {
      setIsoCenter([lat, lon]);
      calculateIsochrones();
      return;
    }

    if (!startLatLng) {
      setStartPoint([lat, lon]);
    } else if (!endLatLng) {
      setEndPoint([lat, lon]);
      calculateRoute();
    } else {
      setEndPoint([lat, lon]);
      calculateRoute();
    }
  });

  function setIsoCenter(latlng) {
    isoCenterLatLng = latlng;
    if (isoCoordsDisplay) {
      isoCoordsDisplay.textContent = `${latlng[0].toFixed(4)}, ${latlng[1].toFixed(4)}`;
    }

    if (isoCenterMarker) {
      isoCenterMarker.setLatLng(latlng);
    } else {
      isoCenterMarker = L.marker(latlng, {
        icon: createPinIcon('center-pin'),
        draggable: true
      }).addTo(map);

      isoCenterMarker.on('dragend', (e) => {
        const p = e.target.getLatLng();
        isoCenterLatLng = [parseFloat(p.lat.toFixed(5)), parseFloat(p.lng.toFixed(5))];
        if (isoCoordsDisplay) {
          isoCoordsDisplay.textContent = `${isoCenterLatLng[0].toFixed(4)}, ${isoCenterLatLng[1].toFixed(4)}`;
        }
        calculateIsochrones();
      });
    }
  }

  // Preset Hub Selection
  if (isoPresetSelect) {
    isoPresetSelect.addEventListener('change', (e) => {
      const city = e.target.value;
      if (city && CITY_HUBS[city]) {
        setIsoCenter(CITY_HUBS[city]);
        map.setView(CITY_HUBS[city], 11);
        calculateIsochrones();
      }
    });
  }

  // Threshold Chips Selection
  chipBtns.forEach((chip) => {
    chip.addEventListener('click', () => {
      const mins = parseInt(chip.getAttribute('data-minutes'), 10);
      if (chip.classList.contains('active')) {
        if (isoBuckets.length > 1) {
          chip.classList.remove('active');
          isoBuckets = isoBuckets.filter((m) => m !== mins);
        }
      } else {
        chip.classList.add('active');
        isoBuckets.push(mins);
        isoBuckets.sort((a, b) => a - b);
      }
      calculateIsochrones();
    });
  });

  if (calcIsoBtn) {
    calcIsoBtn.addEventListener('click', calculateIsochrones);
  }

  if (clearIsoBtn) {
    clearIsoBtn.addEventListener('click', () => {
      isoLayerGroup.clearLayers();
      if (isoResultsCard) isoResultsCard.classList.add('hidden');
    });
  }

  async function calculateIsochrones() {
    if (!isoCenterLatLng || isoBuckets.length === 0) return;

    if (calcIsoBtn) {
      calcIsoBtn.disabled = true;
      calcIsoBtn.textContent = 'Calculating...';
    }

    try {
      const res = await fetch('/api/isochrone', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lat: isoCenterLatLng[0],
          lon: isoCenterLatLng[1],
          buckets: isoBuckets,
          departure_time: selectedDepartureTime || undefined
        })
      });

      if (!res.ok) {
        const err = await res.json();
        alert(err.error || 'Failed to compute reachability contours');
        return;
      }

      const geojson = await res.json();
      displayIsochrones(geojson);
    } catch (e) {
      console.error('Isochrone error:', e);
      alert('Failed to connect to isochrone service');
    } finally {
      if (calcIsoBtn) {
        calcIsoBtn.disabled = false;
        calcIsoBtn.innerHTML = `
          <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
          Compute Reachability
        `;
      }
    }
  }

  function displayIsochrones(geojson) {
    isoLayerGroup.clearLayers();
    if (!geojson.features || geojson.features.length === 0) return;

    // Sort descending by time so largest outer polygon renders below smaller inner polygons
    geojson.features.sort((a, b) => b.properties.time_minutes - a.properties.time_minutes);

    const layer = L.geoJSON(geojson, {
      style: (feature) => {
        const p = feature.properties;
        return {
          color: p.color,
          weight: 2,
          opacity: 0.9,
          fillColor: p.color,
          fillOpacity: 0.28
        };
      },
      onEachFeature: (feature, l) => {
        const p = feature.properties;
        l.bindTooltip(
          `⏱️ <strong>${p.time_minutes} min</strong> contour<br>Area: ${p.area_sq_km} km²<br>Nodes reached: ${p.nodes_reached.toLocaleString()}`,
          { sticky: true }
        );
      }
    }).addTo(isoLayerGroup);

    map.fitBounds(layer.getBounds(), { padding: [50, 50] });

    // Populate legend
    if (isoLegendList) {
      isoLegendList.innerHTML = '';
      const ascending = [...geojson.features].sort((a, b) => a.properties.time_minutes - b.properties.time_minutes);
      ascending.forEach((f) => {
        const p = f.properties;
        const row = document.createElement('div');
        row.className = 'iso-legend-row';
        row.innerHTML = `
          <span class="iso-badge"><span class="iso-color-dot" style="background:${p.color}"></span> ${p.time_minutes} min</span>
          <span><strong>${p.area_sq_km} km²</strong> <span style="color:var(--text-muted);font-size:0.75rem;">(${p.nodes_reached.toLocaleString()} nodes)</span></span>
        `;
        isoLegendList.appendChild(row);
      });
      if (isoResultsCard) isoResultsCard.classList.remove('hidden');
    }

    if (window.innerWidth <= 640) {
      togglePanel(true);
    }
  }

  function setStartPoint(latlng) {
    startLatLng = latlng;
    startCoordsDisplay.textContent = `${latlng[0].toFixed(4)}, ${latlng[1].toFixed(4)}`;

    if (startMarker) {
      startMarker.setLatLng(latlng);
    } else {
      startMarker = L.marker(latlng, {
        icon: createPinIcon('start-pin'),
        draggable: true
      }).addTo(map);

      startMarker.on('dragend', (e) => {
        const p = e.target.getLatLng();
        startLatLng = [parseFloat(p.lat.toFixed(5)), parseFloat(p.lng.toFixed(5))];
        startCoordsDisplay.textContent = `${startLatLng[0].toFixed(4)}, ${startLatLng[1].toFixed(4)}`;
        if (endLatLng) calculateRoute();
      });
    }

    updateButtonStates();
  }

  function setEndPoint(latlng) {
    endLatLng = latlng;
    endCoordsDisplay.textContent = `${latlng[0].toFixed(4)}, ${latlng[1].toFixed(4)}`;

    if (endMarker) {
      endMarker.setLatLng(latlng);
    } else {
      endMarker = L.marker(latlng, {
        icon: createPinIcon('end-pin'),
        draggable: true
      }).addTo(map);

      endMarker.on('dragend', (e) => {
        const p = e.target.getLatLng();
        endLatLng = [parseFloat(p.lat.toFixed(5)), parseFloat(p.lng.toFixed(5))];
        endCoordsDisplay.textContent = `${endLatLng[0].toFixed(4)}, ${endLatLng[1].toFixed(4)}`;
        if (startLatLng) calculateRoute();
      });
    }

    updateButtonStates();
  }

  function updateButtonStates() {
    const ready = startLatLng && endLatLng;
    calcRouteBtn.disabled = !ready;
    compareBtn.disabled = !ready;
  }

  // -------------------------------------------------------------------------
  // Preset Selection
  // -------------------------------------------------------------------------
  presetSelect.addEventListener('change', (e) => {
    const key = e.target.value;
    if (!key || !PRESETS[key]) return;

    const preset = PRESETS[key];
    setStartPoint(preset.start);
    setEndPoint(preset.end);

    map.fitBounds([preset.start, preset.end], { padding: [60, 60] });
    calculateRoute();
  });

  // -------------------------------------------------------------------------
  // Algorithm Card Selection
  // -------------------------------------------------------------------------
  function updateTrafficControlState() {
    const isCH = selectedAlgorithm === 'contraction_hierarchies';
    if (trafficTimeSelect) {
      trafficTimeSelect.disabled = isCH;
      trafficTimeSelect.style.opacity = isCH ? '0.5' : '1.0';
    }
    if (chTrafficNote) {
      if (isCH) {
        chTrafficNote.classList.remove('hidden');
      } else {
        chTrafficNote.classList.add('hidden');
      }
    }
  }

  algoCards.forEach((card) => {
    card.addEventListener('click', () => {
      algoCards.forEach((c) => c.classList.remove('active'));
      card.classList.add('active');
      const radio = card.querySelector('input');
      radio.checked = true;
      selectedAlgorithm = radio.value;

      updateTrafficControlState();

      if (startLatLng && endLatLng) {
        calculateRoute();
      }
    });
  });

  // -------------------------------------------------------------------------
  // Calculate Route API Call
  // -------------------------------------------------------------------------
  async function calculateRoute() {
    if (!startLatLng || !endLatLng) return;

    calcRouteBtn.disabled = true;
    calcRouteBtn.textContent = 'Calculating...';

    const wantsWavefront = showWavefrontToggle ? showWavefrontToggle.checked : true;
    const depTime = selectedAlgorithm === 'contraction_hierarchies' ? undefined : (selectedDepartureTime || undefined);

    try {
      const res = await fetch('/api/route', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          start_lat: startLatLng[0],
          start_lon: startLatLng[1],
          end_lat: endLatLng[0],
          end_lon: endLatLng[1],
          algorithm: selectedAlgorithm,
          metric: selectedMetric,
          departure_time: depTime,
          include_explored: wantsWavefront
        })
      });

      if (!res.ok) {
        const err = await res.json();
        alert(err.error || 'No route found between these points');
        return;
      }

      const data = await res.json();
      displayRoute(data);
    } catch (err) {
      console.error('Route calculation error:', err);
      alert('Failed to connect to pathfinding service.');
    } finally {
      calcRouteBtn.disabled = false;
      calcRouteBtn.innerHTML = `
        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><polyline points="9 18 15 12 9 6"></polyline></svg>
        Calculate Route
      `;
    }
  }

  calcRouteBtn.addEventListener('click', calculateRoute);

  function displayRoute(data) {
    // GeoJSON coordinates come in [lon, lat] format
    const latlngs = data.path.map((coord) => [coord[1], coord[0]]);

    if (routePolyline) {
      map.removeLayer(routePolyline);
    }

    routePolyline = L.polyline(latlngs, {
      color: '#06b6d4',
      weight: 5,
      opacity: 0.95,
      lineJoin: 'round'
    }).addTo(map);

    // Render search wavefront exploration dots
    wavefrontLayer.clearLayers();
    if (showWavefrontToggle && showWavefrontToggle.checked && data.explored && data.explored.length > 0) {
      const dotColor = selectedAlgorithm.includes('dijkstra') ? '#f59e0b' : '#38bdf8';
      data.explored.forEach((pt) => {
        L.circleMarker([pt[1], pt[0]], {
          radius: 3.5,
          color: dotColor,
          fillColor: dotColor,
          fillOpacity: 0.45,
          weight: 0
        }).addTo(wavefrontLayer);
      });
    }

    // Zoom to fit path
    map.fitBounds(routePolyline.getBounds(), { padding: [50, 50] });

    // Format metrics
    const distKm = (data.distance_m / 1000).toFixed(1);
    const mins = Math.round(data.duration_s / 60);
    const hours = Math.floor(mins / 60);
    const remainingMins = mins % 60;
    const durationText = hours > 0 ? `${hours}h ${remainingMins}m` : `${mins} min`;

    metricDistance.textContent = `${distKm} km`;
    metricDuration.textContent = durationText;
    metricVisited.textContent = data.nodes_visited.toLocaleString();
    metricLatency.textContent = data.query_time_ms < 1
      ? `${(data.query_time_ms * 1000).toFixed(0)} μs`
      : `${data.query_time_ms.toFixed(2)} ms`;

    metricsCard.classList.remove('hidden');

    if (graphBadge) {
      graphBadge.textContent = `${distKm} km • ${durationText}`;
      graphBadge.classList.add('ready');
    }

    if (window.innerWidth <= 640) {
      togglePanel(true);
    }
  }

  // -------------------------------------------------------------------------
  // Side-by-Side Algorithm Benchmark (Compare All 4)
  // -------------------------------------------------------------------------
  compareBtn.addEventListener('click', async () => {
    if (!startLatLng || !endLatLng) return;

    compareBtn.disabled = true;
    compareBtn.textContent = 'Benchmarking...';

    const algorithms = [
      { id: 'contraction_hierarchies', name: '⚡ Contraction Hierarchies (Static)' },
      { id: 'astar', name: 'A* Search' },
      { id: 'bidirectional_astar', name: 'Bi-directional A*' },
      { id: 'dijkstra', name: 'Dijkstra' },
      { id: 'bidirectional_dijkstra', name: 'Bi-directional Dijkstra' }
    ];

    compareTbody.innerHTML = '';
    compareCard.classList.remove('hidden');

    const results = [];

    for (const algo of algorithms) {
      try {
        const res = await fetch('/api/route', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            start_lat: startLatLng[0],
            start_lon: startLatLng[1],
            end_lat: endLatLng[0],
            end_lon: endLatLng[1],
            algorithm: algo.id,
            metric: selectedMetric,
            departure_time: algo.id === 'contraction_hierarchies' ? undefined : (selectedDepartureTime || undefined),
            include_explored: false
          })
        });

        if (res.ok) {
          const data = await res.json();
          results.push({ ...data, displayName: algo.name });
        }
      } catch (e) {
        console.error('Benchmark failed for', algo.id, e);
      }
    }

    // Populate comparison rows
    results.forEach((r) => {
      const tr = document.createElement('tr');
      const timeStr = r.query_time_ms < 1
        ? `${(r.query_time_ms * 1000).toFixed(0)} μs`
        : `${r.query_time_ms.toFixed(2)} ms`;

      tr.innerHTML = `
        <td class="algo-name">${r.displayName}</td>
        <td><strong>${r.nodes_visited.toLocaleString()}</strong></td>
        <td>${timeStr}</td>
        <td>${(r.distance_m / 1000).toFixed(1)} km</td>
      `;
      compareTbody.appendChild(tr);
    });

    // Provide mathematical insight for academic report
    const dijkstraResult = results.find((r) => r.algorithm === 'dijkstra');
    const astarResult = results.find((r) => r.algorithm === 'astar');
    const biAstarResult = results.find((r) => r.algorithm === 'bidirectional_astar');
    const chResult = results.find((r) => r.algorithm === 'contraction_hierarchies');

    if (dijkstraResult && astarResult && dijkstraResult.nodes_visited > 0) {
      const reduction = (
        ((dijkstraResult.nodes_visited - astarResult.nodes_visited) /
          dijkstraResult.nodes_visited) *
        100
      ).toFixed(1);

      let insight = `✨ <strong>Heuristic Pruning:</strong> A* examined <strong>${reduction}%</strong> fewer nodes than Dijkstra while yielding identical optimal distance.`;

      if (biAstarResult && biAstarResult.nodes_visited < astarResult.nodes_visited) {
        const biReduction = (
          ((astarResult.nodes_visited - biAstarResult.nodes_visited) /
            astarResult.nodes_visited) *
          100
        ).toFixed(1);
        insight += `<br>🚀 <strong>Dual Wavefronts:</strong> Bi-directional A* further reduced search space by another <strong>${biReduction}%</strong>.`;
      }

      if (chResult && dijkstraResult.nodes_visited > 0) {
        const chReduction = (
          ((dijkstraResult.nodes_visited - chResult.nodes_visited) /
            dijkstraResult.nodes_visited) *
          100
        ).toFixed(1);
        insight += `<br>⚡ <strong>Contraction Hierarchies:</strong> Hierarchical upward search pruned <strong>${chReduction}%</strong> of search space, delivering microsecond responses!`;
      }

      compareInsight.innerHTML = insight;
    }

    compareBtn.disabled = false;
    compareBtn.textContent = 'Compare All 5';
  });

  // -------------------------------------------------------------------------
  // Clear State
  // -------------------------------------------------------------------------
  clearBtn.addEventListener('click', () => {
    startLatLng = null;
    endLatLng = null;

    if (startMarker) {
      map.removeLayer(startMarker);
      startMarker = null;
    }
    if (endMarker) {
      map.removeLayer(endMarker);
      endMarker = null;
    }
    if (routePolyline) {
      map.removeLayer(routePolyline);
      routePolyline = null;
    }
    wavefrontLayer.clearLayers();

    startCoordsDisplay.textContent = 'Click map or pick preset';
    endCoordsDisplay.textContent = 'Click map or pick preset';
    presetSelect.value = '';

    metricsCard.classList.add('hidden');
    compareCard.classList.add('hidden');

    updateButtonStates();
  });
});
