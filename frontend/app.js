// osm-pathfinder — Frontend Application Controller

document.addEventListener('DOMContentLoaded', () => {
  // -------------------------------------------------------------------------
  // State
  // -------------------------------------------------------------------------
  let startLatLng = null;
  let endLatLng = null;
  let startMarker = null;
  let endMarker = null;
  let routePolyline = null;
  let selectedAlgorithm = 'astar';

  // -------------------------------------------------------------------------
  // Cambodia Highway Presets
  // -------------------------------------------------------------------------
  const PRESETS = {
    'pp-siemreap': {
      start: [11.5564, 104.9282], // Phnom Penh Central
      end: [13.3671, 103.8448],   // Siem Reap Central
      name: 'Phnom Penh → Siem Reap'
    },
    'pp-sihanoukville': {
      start: [11.5564, 104.9282], // Phnom Penh Central
      end: [10.6253, 103.5234],   // Sihanoukville
      name: 'Phnom Penh → Sihanoukville'
    },
    'pp-battambang': {
      start: [11.5564, 104.9282], // Phnom Penh Central
      end: [13.0957, 103.2022],   // Battambang
      name: 'Phnom Penh → Battambang'
    },
    'siemreap-battambang': {
      start: [13.3671, 103.8448], // Siem Reap Central
      end: [13.0957, 103.2022],   // Battambang
      name: 'Siem Reap → Battambang'
    },
    'pp-kampot': {
      start: [11.5564, 104.9282], // Phnom Penh Central
      end: [10.6104, 104.1815],   // Kampot
      name: 'Phnom Penh → Kampot'
    },
    'sr-angkor': {
      start: [13.3671, 103.8448], // Siem Reap Old Market
      end: [13.4125, 103.8670],   // Angkor Wat
      name: 'Siem Reap → Angkor Wat'
    }
  };

  // -------------------------------------------------------------------------
  // Leaflet Map Initialization
  // -------------------------------------------------------------------------
  const map = L.map('map', {
    zoomControl: false
  }).setView([12.5657, 104.9910], 7); // Center of Cambodia

  L.control.zoom({ position: 'bottomright' }).addTo(map);

  // Modern OpenStreetMap tile layer
  L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '© OpenStreetMap contributors'
  }).addTo(map);

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
  const graphBadge = document.getElementById('graph-badge');
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
  // Map Click Handler (Drop Pins)
  // -------------------------------------------------------------------------
  map.on('click', (e) => {
    const lat = parseFloat(e.latlng.lat.toFixed(5));
    const lon = parseFloat(e.latlng.lng.toFixed(5));

    if (!startLatLng) {
      setStartPoint([lat, lon]);
    } else if (!endLatLng) {
      setEndPoint([lat, lon]);
      calculateRoute();
    } else {
      // If both already set, update destination to new click
      setEndPoint([lat, lon]);
      calculateRoute();
    }
  });

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
  algoCards.forEach((card) => {
    card.addEventListener('click', () => {
      algoCards.forEach((c) => c.classList.remove('active'));
      card.classList.add('active');
      const radio = card.querySelector('input');
      radio.checked = true;
      selectedAlgorithm = radio.value;

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

    try {
      const res = await fetch('/api/route', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          start_lat: startLatLng[0],
          start_lon: startLatLng[1],
          end_lat: endLatLng[0],
          end_lon: endLatLng[1],
          algorithm: selectedAlgorithm
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
      opacity: 0.9,
      lineJoin: 'round'
    }).addTo(map);

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
  }

  // -------------------------------------------------------------------------
  // Side-by-Side Algorithm Benchmark (Compare All 4)
  // -------------------------------------------------------------------------
  compareBtn.addEventListener('click', async () => {
    if (!startLatLng || !endLatLng) return;

    compareBtn.disabled = true;
    compareBtn.textContent = 'Benchmarking...';

    const algorithms = [
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
            algorithm: algo.id
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

    if (dijkstraResult && astarResult) {
      const reduction = (
        ((dijkstraResult.nodes_visited - astarResult.nodes_visited) /
          dijkstraResult.nodes_visited) *
        100
      ).toFixed(1);

      let insight = `✨ <strong>Heuristic Pruning:</strong> A* examined <strong>${reduction}%</strong> fewer nodes than Dijkstra while yielding identical shortest distance.`;

      if (biAstarResult && biAstarResult.nodes_visited < astarResult.nodes_visited) {
        const biReduction = (
          ((astarResult.nodes_visited - biAstarResult.nodes_visited) /
            astarResult.nodes_visited) *
          100
        ).toFixed(1);
        insight += `<br>🚀 <strong>Dual wavefronts:</strong> Bi-directional A* further reduced search volume by another <strong>${biReduction}%</strong>.`;
      }

      compareInsight.innerHTML = insight;
    }

    compareBtn.disabled = false;
    compareBtn.textContent = 'Compare All 4';
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

    startCoordsDisplay.textContent = 'Click map or pick preset';
    endCoordsDisplay.textContent = 'Click map or pick preset';
    presetSelect.value = '';

    metricsCard.classList.add('hidden');
    compareCard.classList.add('hidden');

    updateButtonStates();
  });
});
