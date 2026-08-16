export function createDungeonLoadUi({ wasm }) {
  let selectedEquipmentSlot = 'weapon';
  let lastPlayerTile = -1;
  let logLineCount = 5;
  try {
    const savedLogLineCount = Number(localStorage.getItem('dungeonload-log-lines'));
    if (Number.isInteger(savedLogLineCount) && savedLogLineCount >= 3 && savedLogLineCount <= 8) {
      logLineCount = savedLogLineCount;
    }
  } catch (_) {
    // localStorage may be unavailable in private browsing contexts.
  }
  let controlsSwapped = false;
  try {
    controlsSwapped = localStorage.getItem('dungeonload-swap-controls') === 'true';
  } catch (_) {
    // Keep the default side when storage is unavailable.
  }

  function setControlsSwapped(enabled) {
    controlsSwapped = Boolean(enabled);
    document.body.classList.toggle('controls-swapped', controlsSwapped);
    try {
      localStorage.setItem('dungeonload-swap-controls', String(controlsSwapped));
    } catch (_) {
      // Keep the setting for this session if storage is unavailable.
    }
  }

  setControlsSwapped(controlsSwapped);

  const downRotation = { x: 0, y: 160, z: 0 };
  let downPreviewRenderer = null;
  let downPreviewScene = null;
  let downPreviewCamera = null;
  let downPreviewMesh = null;
  ['x', 'y', 'z'].forEach((axis) => {
    try {
      const saved = Number(localStorage.getItem(`dungeonload-down-rotation-${axis}`));
      if (Number.isFinite(saved) && saved >= -180 && saved <= 180) downRotation[axis] = saved;
    } catch (_) {
      // Keep the default rotation when storage is unavailable.
    }
  });

  function getDownRotation() {
    return { ...downRotation };
  }

  function renderDownPreview() {
    if (!downPreviewRenderer || !downPreviewMesh) return;
    downPreviewMesh.rotation.set(
      downRotation.x * Math.PI / 180,
      downRotation.y * Math.PI / 180,
      downRotation.z * Math.PI / 180,
    );
    downPreviewRenderer.render(downPreviewScene, downPreviewCamera);
  }

  function setupDownPreview(panel) {
    const canvas = panel.querySelector('#down-rotation-preview');
    const three = window.THREE;
    if (!canvas || !three) return;
    const image = panel.querySelector('#down-rotation-preview-image');
    const width = Math.max(160, canvas.clientWidth || 320);
    const height = 126;
    downPreviewRenderer?.dispose();
    downPreviewRenderer = new three.WebGLRenderer({ canvas, alpha: true, antialias: true });
    downPreviewRenderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    downPreviewRenderer.setSize(width, height, false);
    downPreviewScene = new three.Scene();
    downPreviewCamera = new three.PerspectiveCamera(75, width / height, 0.1, 100);
    downPreviewCamera.position.set(0, 6, 6);
    downPreviewCamera.lookAt(0, 0, -2);
    const texture = new three.Texture(image);
    texture.needsUpdate = true;
    const material = new three.MeshBasicMaterial({ map: texture, transparent: true, side: three.DoubleSide });
    downPreviewMesh = new three.Mesh(new three.PlaneGeometry(4.5, 4.5), material);
    downPreviewScene.add(downPreviewMesh);
    renderDownPreview();
    const refreshTexture = () => {
      texture.needsUpdate = true;
      renderDownPreview();
    };
    if (image.complete && image.naturalWidth > 0) refreshTexture();
    else image.addEventListener('load', refreshTexture, { once: true });
  }

  function updateStatusTab(panel) {
    if (!panel) return;
    const level = wasm.level_roguelike();
    const depth = wasm.depth_roguelike();
    const hp = wasm.hp_roguelike();
    const maxHp = wasm.max_hp_roguelike();
    const mp = wasm.mp_roguelike();
    const maxMp = wasm.max_mp_roguelike();
    const atkBonus = wasm.player_atk_roguelike();
    const defBonus = wasm.player_def_roguelike();
    const hpPercent = Math.round((hp / maxHp) * 100);
    const mpPercent = Math.round((mp / maxMp) * 100);

    let html = '<div style="padding: 16px; display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px;">';
    html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">📊</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">LEVEL</div>
      <div style="font-size: 20px; color: #0ff; font-weight: bold;">${level}</div>
    </div>`;
    html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">🗻</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">DEPTH</div>
      <div style="font-size: 20px; color: #0ff; font-weight: bold;">F${depth}</div>
    </div>`;
    html += `<div style="border: 1px solid rgba(255,0,0,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">❤️</div>
      <div style="font-size: 12px; color: rgba(255,100,100,0.7); margin-bottom: 4px;">HP</div>
      <div style="font-size: 14px; color: #f0f; font-weight: bold;">${hp}/${maxHp}</div>
      <div style="width: 100%; height: 4px; background: rgba(255,0,0,0.2); border-radius: 2px; margin-top: 4px;">
        <div style="width: ${hpPercent}%; height: 100%; background: #f00; border-radius: 2px;"></div>
      </div>
    </div>`;
    html += `<div style="border: 1px solid rgba(0,100,255,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">💎</div>
      <div style="font-size: 12px; color: rgba(0,100,255,0.7); margin-bottom: 4px;">MP</div>
      <div style="font-size: 14px; color: #00f; font-weight: bold;">${mp}/${maxMp}</div>
      <div style="width: 100%; height: 4px; background: rgba(0,100,255,0.2); border-radius: 2px; margin-top: 4px;">
        <div style="width: ${mpPercent}%; height: 100%; background: #00f; border-radius: 2px;"></div>
      </div>
    </div>`;
    html += `<div style="border: 1px solid rgba(255,200,0,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">⚔️</div>
      <div style="font-size: 12px; color: rgba(255,200,0,0.7); margin-bottom: 4px;">ATTACK</div>
      <div style="font-size: 20px; color: #fd0; font-weight: bold;">+${atkBonus}</div>
    </div>`;
    html += `<div style="border: 1px solid rgba(150,150,150,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">🛡️</div>
      <div style="font-size: 12px; color: rgba(150,150,150,0.7); margin-bottom: 4px;">DEFENSE</div>
      <div style="font-size: 20px; color: #aaa; font-weight: bold;">+${defBonus}</div>
    </div>`;
    html += '</div>';
    panel.innerHTML = html;
  }

  function updateEquipmentTab(panel) {
    if (!panel) return;
    const weaponNames = ['Wooden Sword', 'Iron Sword', 'Spear', 'Bow', 'Staff', 'Cursed Blade'];
    const armorNames = ['Leather Armor', 'Chain Mail', 'Steel Plate', 'Dragon Scale', 'Cursed Mail'];
    const accessoryNames = ['Gold Ring', 'Vampire Ring', 'Lucky Ring', 'Healing Necklace', 'Mana Earrings'];
    const weaponBonuses = [3, 5, 7, 9, 8, 12];
    const armorBonuses = [2, 4, 6, 8, 10];
    const wepIdx = wasm.player_equipped_weapon_roguelike();
    const armIdx = wasm.player_equipped_armor_roguelike();
    const accIdx = wasm.player_equipped_accessory_roguelike();
    let html = '<div style="padding: 16px;">';
    html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-bottom: 16px;">';
    html += `<div style="border: 2px solid ${selectedEquipmentSlot === 'weapon' ? '#0ff' : 'rgba(0,200,255,0.3)'}; border-radius: 4px; padding: 12px; text-align: center; cursor: pointer;" onclick="selectEquipmentSlot('weapon', event)">
      <div style="font-size: 24px; margin-bottom: 4px;">⚔️</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">WEAPON</div>
      <div style="font-size: 11px; color: #0ff;">${wepIdx >= 0 ? weaponNames[wepIdx] : 'None'}</div>
    </div>`;
    html += `<div style="border: 2px solid ${selectedEquipmentSlot === 'armor' ? '#0ff' : 'rgba(0,200,255,0.3)'}; border-radius: 4px; padding: 12px; text-align: center; cursor: pointer;" onclick="selectEquipmentSlot('armor', event)">
      <div style="font-size: 24px; margin-bottom: 4px;">🛡️</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">ARMOR</div>
      <div style="font-size: 11px; color: #0ff;">${armIdx >= 0 ? armorNames[armIdx] : 'None'}</div>
    </div>`;
    html += `<div style="border: 2px solid ${selectedEquipmentSlot === 'accessory' ? '#0ff' : 'rgba(0,200,255,0.3)'}; border-radius: 4px; padding: 12px; text-align: center; cursor: pointer;" onclick="selectEquipmentSlot('accessory', event)">
      <div style="font-size: 24px; margin-bottom: 4px;">💍</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">ACCESSORY</div>
      <div style="font-size: 11px; color: #0ff;">${accIdx >= 0 ? accessoryNames[accIdx] : 'None'}</div>
    </div>`;
    html += '</div>';
    if (selectedEquipmentSlot === 'weapon') {
      const weapons = wasm.weapon_inventory_roguelike();
      html += '<div style="border-top: 1px solid rgba(0,200,255,0.2); padding-top: 12px;"><div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 8px;">AVAILABLE WEAPONS</div>';
      html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;">';
      for (let i = 0; i < weapons.length; i++) {
        html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px;" onclick="wasm.equip_weapon_roguelike(${i}); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">
          <div>${weaponNames[weapons[i]]}</div>
          <div style="color: rgba(0,200,255,0.5); font-size: 9px;">+${weaponBonuses[weapons[i]]}</div>
        </div>`;
      }
      html += `<div style="border: 1px solid rgba(200,0,0,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px; background: rgba(200,0,0,0.1);" onclick="wasm.unequip_weapon_roguelike(); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">Unequip</div>`;
      html += '</div></div>';
    } else if (selectedEquipmentSlot === 'armor') {
      const armors = wasm.armor_inventory_roguelike();
      html += '<div style="border-top: 1px solid rgba(0,200,255,0.2); padding-top: 12px;"><div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 8px;">AVAILABLE ARMOR</div>';
      html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;">';
      for (let i = 0; i < armors.length; i++) {
        html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px;" onclick="wasm.equip_armor_roguelike(${i}); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">
          <div>${armorNames[armors[i]]}</div>
          <div style="color: rgba(0,200,255,0.5); font-size: 9px;">+${armorBonuses[armors[i]]}</div>
        </div>`;
      }
      html += `<div style="border: 1px solid rgba(200,0,0,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px; background: rgba(200,0,0,0.1);" onclick="wasm.unequip_armor_roguelike(); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">Unequip</div>`;
      html += '</div></div>';
    } else if (selectedEquipmentSlot === 'accessory') {
      const accessories = wasm.accessory_inventory_roguelike();
      html += '<div style="border-top: 1px solid rgba(0,200,255,0.2); padding-top: 12px;"><div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 8px;">AVAILABLE ACCESSORIES</div>';
      html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;">';
      for (let i = 0; i < accessories.length; i++) {
        html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px;" onclick="wasm.equip_accessory_roguelike(${i}); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">
          <div>${accessoryNames[accessories[i]]}</div>
        </div>`;
      }
      html += `<div style="border: 1px solid rgba(200,0,0,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px; background: rgba(200,0,0,0.1);" onclick="wasm.unequip_accessory_roguelike(); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">Unequip</div>`;
      html += '</div></div>';
    }
    html += '</div>';
    panel.innerHTML = html;
  }

  function selectEquipmentSlot(slot, event) {
    if (event) event.stopPropagation();
    selectedEquipmentSlot = slot;
    updateEquipmentTab(document.getElementById('tab-equipment'));
  }

  function updateEquipmentDisplay() {
    updateEquipmentTab(document.getElementById('tab-equipment'));
  }

  function updateItemsDisplay() {
    const panel = document.getElementById('tab-items');
    if (!panel) return;
    const inventory = wasm.inventory_roguelike();
    const itemNames = ['Health Potion', 'Mana Potion', 'Poison Potion', 'Energy Drink', 'Gem', 'Key', 'Scroll', 'Gold Coin'];
    const itemEmojis = ['💚', '💙', '☠️', '⚡', '💎', '🔑', '📜', '🪙'];
    let itemsHtml = '<div style="padding: 16px; display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px;">';
    for (let i = 0; i < Math.min(inventory.length, itemNames.length); i++) {
      if (inventory[i] > 0 || i < 5) {
        itemsHtml += `
          <div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 8px; text-align: center;">
            <div style="font-size: 24px; margin-bottom: 4px;">${itemEmojis[i]}</div>
            <div style="font-size: 11px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">${itemNames[i]}</div>
            <div style="font-size: 14px; color: #0ff; font-weight: bold;">×${inventory[i] || 0}</div>
          </div>
        `;
      }
    }
    itemsHtml += '</div>';
    panel.innerHTML = itemsHtml;
  }

  function updateMessageLog() {
    const scene = wasm.scene_roguelike();
    const msgLogEl = document.getElementById('message-log');
    if (scene === 1) {
      msgLogEl.style.display = 'block';
      const messages = wasm.messages_roguelike();
      msgLogEl.style.maxHeight = `${logLineCount * 1.4 + 1.5}em`;
      msgLogEl.innerHTML = messages
        .map(msg => '<div style="margin: 2px 0; word-break: break-word;">' + msg + '</div>')
        .join('');
      msgLogEl.scrollTop = msgLogEl.scrollHeight;
    } else {
      msgLogEl.style.display = 'none';
    }
  }

  function updateLogTab(panel) {
    if (!panel) return;
    const messages = wasm.messages_roguelike();
    panel.innerHTML = `
      <div style="padding: 16px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 10px;">
        <div class="stat-label" style="font-size: 1.1em;">MESSAGE LOG</div>
        <div style="border: 1px solid rgba(0,200,255,0.35); padding: 10px; overflow-y: auto; flex: 1; min-height: 0; line-height: 1.5; color: #aaf; text-shadow: 0 0 4px #0ff;">
          ${messages.map(msg => `<div style="margin: 3px 0; word-break: break-word;">${msg}</div>`).join('') || '<div style="color: #888;">No messages yet.</div>'}
        </div>
      </div>`;
    const logBox = panel.querySelector('div[style*="overflow-y"]');
    if (logBox) logBox.scrollTop = logBox.scrollHeight;
  }

  function flashHint(action) {
    const hints = ['sh-up', 'sh-left', 'sh-right', 'sh-down'];
    const hint = document.querySelector(`.${hints[action]}`);
    if (!hint) return;
    hint.classList.add('flash');
    setTimeout(() => {
      hint.classList.remove('flash');
    }, 200);
  }

  function setupNavigation() {
    const navBtns = document.querySelectorAll('.nav-btn[data-tab]');
    const uiUpper = document.getElementById('ui-upper');
    const closePanel = () => {
      uiUpper?.classList.remove('panel-open');
      navBtns.forEach(b => b.classList.remove('active'));
    };
    document.getElementById('panel-close')?.addEventListener('click', closePanel);
    navBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        const tabName = btn.getAttribute('data-tab');
        if (btn.classList.contains('active') && uiUpper?.classList.contains('panel-open')) {
          closePanel();
          return;
        }
        navBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        uiUpper?.classList.add('panel-open');
        document.querySelectorAll('.tab-panel').forEach(panel => panel.classList.remove('active'));
        const tabPanel = document.getElementById(`tab-${tabName}`);
        if (tabPanel) tabPanel.classList.add('active');
        updateTabContent(tabName);
        if (tabName === 'map') setTimeout(() => drawDungeonMap(), 100);
      });
    });
  }

  function updateTabContent(tabName) {
    const panel = document.getElementById(`tab-${tabName}`);
    switch (tabName) {
      case 'status':
        updateStatusTab(panel);
        break;
      case 'equipment':
        updateEquipmentTab(panel);
        break;
      case 'items':
        updateItemsDisplay();
        break;
      case 'settings':
        updateSettingsTab(panel);
        break;
      case 'log':
        updateLogTab(panel);
        break;
      case 'map':
        drawDungeonMap();
        break;
    }
  }

  function updateSettingsTab(panel) {
    if (!panel) return;
    downPreviewRenderer?.dispose();
    downPreviewRenderer = null;
    downPreviewMesh = null;
    const scale = wasm.enemy_attack_interval_scale_roguelike();
    const noDamageMode = wasm.no_damage_mode_roguelike();
    const depth = wasm.depth_roguelike();
    const rotation = getDownRotation();
    panel.innerHTML = `
      <div style="padding: 16px;">
        <div class="stat-item" style="display: block;">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px;">
            <span class="stat-label">LOG LINES</span>
            <span class="stat-value" id="log-lines-value">${logLineCount}</span>
          </div>
          <input id="log-lines-slider" type="range" min="3" max="8" step="1" value="${logLineCount}" style="width: 100%; margin-top: 14px; accent-color: #0ff;">
          <div style="color: rgba(180,220,240,0.75); font-size: 0.75em; line-height: 1.5; margin-top: 8px;">
            Choose how many log rows are visible at once. Scroll to view older messages.
          </div>
        </div>

        <div class="stat-item" style="display: block;">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px;">
            <span class="stat-label">ENEMY ATTACK INTERVAL</span>
            <span class="stat-value" id="enemy-interval-value">${scale}%</span>
          </div>
          <input id="enemy-interval-slider" type="range" min="50" max="1000" step="10" value="${scale}" style="width: 100%; margin-top: 14px; accent-color: #0ff;">
          <div style="color: rgba(180,220,240,0.75); font-size: 0.75em; line-height: 1.5; margin-top: 8px;">
            Higher values make enemy attacks slower. Differences between enemy types are preserved.
          </div>
        </div>

        <div class="stat-item" style="display: block; margin-top: 18px;">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px;">
            <span class="stat-label">SWAP CONTROL SIDES</span>
            <label style="display: flex; align-items: center; gap: 8px; color: #0ff; cursor: pointer; user-select: none;">
              <input id="swap-controls-toggle" type="checkbox" ${controlsSwapped ? 'checked' : ''} style="accent-color: #0ff;">
              <span id="swap-controls-label">${controlsSwapped ? 'ON' : 'OFF'}</span>
            </label>
          </div>
          <div style="color: rgba(180,220,240,0.75); font-size: 0.75em; line-height: 1.5; margin-top: 8px;">
            Move the attack and dodge buttons to the opposite side.
          </div>
        </div>

        <div class="stat-item" style="display: block; margin-top: 18px;">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px;">
            <span class="stat-label">NO DAMAGE MODE</span>
            <label style="display: flex; align-items: center; gap: 8px; color: #0ff; cursor: pointer; user-select: none;">
              <input id="no-damage-toggle" type="checkbox" ${noDamageMode ? 'checked' : ''} style="accent-color: #0ff;">
              <span id="no-damage-label">${noDamageMode ? 'ON' : 'OFF'}</span>
            </label>
          </div>
          <div style="color: rgba(180,220,240,0.75); font-size: 0.75em; line-height: 1.5; margin-top: 8px;">
            Enemy attacks will not reduce HP while this mode is enabled.
          </div>
        </div>

        <div class="stat-item" style="display: block; margin-top: 18px;">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px;">
            <span class="stat-label">FLOOR JUMP</span>
            <span class="stat-value" id="floor-jump-value">F${depth}</span>
          </div>
          <input id="floor-jump-slider" type="range" min="1" max="30" step="1" value="${depth}" style="width: 100%; margin-top: 14px; accent-color: #0ff;">
          <button id="floor-jump-button" style="width: 100%; margin-top: 10px; padding: 8px 14px; background: rgba(0,200,255,0.15); border: 1px solid #0ff; color: #0ff; cursor: pointer; font-family: inherit;">JUMP TO F${depth}</button>
          <div id="floor-jump-status" style="color: rgba(180,220,240,0.75); font-size: 0.75em; line-height: 1.5; margin-top: 8px;">
            Jump to any floor from F1 to F30.
          </div>
        </div>

        <div class="stat-item" style="display: block; margin-top: 18px;">
          <div class="stat-label">DOWN ICON ROTATION</div>
          <div style="color: rgba(180,220,240,0.75); font-size: 0.75em; line-height: 1.5; margin: 8px 0 14px;">
            Adjust the 3D player icon when facing down.
          </div>
          ${['x', 'y', 'z'].map((axis) => `
            <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-top: 10px;">
              <span class="stat-label">${axis.toUpperCase()} AXIS</span>
              <span class="stat-value" id="down-rotation-${axis}-value">${rotation[axis]}°</span>
            </div>
            <input id="down-rotation-${axis}-slider" type="range" min="-180" max="180" step="1" value="${rotation[axis]}" style="width: 100%; margin-top: 8px; accent-color: #0ff;">
          `).join('')}
          <div style="margin-top: 18px; text-align: center;">
            <div class="stat-label" style="margin-bottom: 8px;">PLAYER PREVIEW</div>
            <div style="height: 126px; display: grid; place-items: center; background: rgba(0,20,32,0.65); border: 1px solid rgba(0,200,255,0.35); border-radius: 4px; perspective: 420px; overflow: hidden;">
              <canvas id="down-rotation-preview" width="320" height="126" style="display: block; width: 100%; height: 126px;"></canvas>
              <img id="down-rotation-preview-image" src="../roguelike/icons/cathelineau/swordman.png" alt="Player preview source" width="1" height="1" style="display: none;">
            </div>
          </div>
        </div>
      </div>`;

    const logLinesSlider = panel.querySelector('#log-lines-slider');
    const logLinesValue = panel.querySelector('#log-lines-value');
    logLinesSlider.addEventListener('input', () => {
      logLineCount = Number(logLinesSlider.value);
      logLinesValue.textContent = String(logLineCount);
      try {
        localStorage.setItem('dungeonload-log-lines', String(logLineCount));
      } catch (_) {
        // Keep the setting for this session if storage is unavailable.
      }
      updateMessageLog();
    });

    const swapControlsToggle = panel.querySelector('#swap-controls-toggle');
    const swapControlsLabel = panel.querySelector('#swap-controls-label');
    swapControlsToggle.addEventListener('change', () => {
      setControlsSwapped(swapControlsToggle.checked);
      swapControlsLabel.textContent = swapControlsToggle.checked ? 'ON' : 'OFF';
    });

    const slider = panel.querySelector('#enemy-interval-slider');
    const value = panel.querySelector('#enemy-interval-value');
    slider.addEventListener('input', () => {
      const nextScale = Number(slider.value);
      wasm.set_enemy_attack_interval_scale_roguelike(nextScale);
      value.textContent = `${nextScale}%`;
    });

    const noDamageToggle = panel.querySelector('#no-damage-toggle');
    const noDamageLabel = panel.querySelector('#no-damage-label');
    noDamageToggle.addEventListener('change', () => {
      const enabled = noDamageToggle.checked;
      wasm.set_no_damage_mode_roguelike(enabled);
      noDamageLabel.textContent = enabled ? 'ON' : 'OFF';
    });

    const floorSlider = panel.querySelector('#floor-jump-slider');
    const floorValue = panel.querySelector('#floor-jump-value');
    const floorButton = panel.querySelector('#floor-jump-button');
    const updateFloorSelection = () => {
      const nextFloor = Number(floorSlider.value);
      floorValue.textContent = `F${nextFloor}`;
      floorButton.textContent = `JUMP TO F${nextFloor}`;
    };
    const jumpToFloor = () => {
      const nextFloor = Number(floorSlider.value);
      wasm.jump_to_floor_roguelike(nextFloor);
      updateTabContent('status');
      updateSettingsTab(panel);
      updateTabContent('map');
    };
    floorSlider.addEventListener('input', updateFloorSelection);
    floorButton.addEventListener('click', jumpToFloor);

    ['x', 'y', 'z'].forEach((axis) => {
      const rotationSlider = panel.querySelector(`#down-rotation-${axis}-slider`);
      const rotationValue = panel.querySelector(`#down-rotation-${axis}-value`);
      rotationSlider.addEventListener('input', () => {
        downRotation[axis] = Number(rotationSlider.value);
        rotationValue.textContent = `${downRotation[axis]}°`;
        try {
          localStorage.setItem(`dungeonload-down-rotation-${axis}`, String(downRotation[axis]));
        } catch (_) {
          // Keep the setting for this session if storage is unavailable.
        }
      });
    });
    ['x', 'y', 'z'].forEach((axis) => {
      panel.querySelector(`#down-rotation-${axis}-slider`).addEventListener('input', renderDownPreview);
    });
    setupDownPreview(panel);
  }

  function drawDungeonMap() {
    const canvas = document.getElementById('dungeon-map');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width;
    canvas.height = rect.height;
    const mapWidth = wasm.map_width_roguelike();
    const mapHeight = wasm.map_height_roguelike();
    const playerX = wasm.player_x_roguelike();
    const playerY = wasm.player_y_roguelike();
    const mapData = wasm.map_data_roguelike();
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const cellW = canvas.width / mapWidth;
    const cellH = canvas.height / mapHeight;
    const visitedData = wasm.visited_data_roguelike();
    let roomTiles = new Set();
    let currentPlayerTile = -1;
    if (playerX >= 0 && playerX < mapWidth && playerY >= 0 && playerY < mapHeight) {
      const playerIdx = playerY * mapWidth + playerX;
      const playerTile = mapData[playerIdx];
      currentPlayerTile = playerTile;
      if (playerTile === 2) {
        const queue = [[playerX, playerY]];
        const visited = new Set();
        visited.add(`${playerX},${playerY}`);
        while (queue.length > 0) {
          const [x, y] = queue.shift();
          roomTiles.add(`${x},${y}`);
          for (const [dx, dy] of [[0, 1], [0, -1], [1, 0], [-1, 0]]) {
            const nx = x + dx;
            const ny = y + dy;
            const key = `${nx},${ny}`;
            if (nx >= 0 && nx < mapWidth && ny >= 0 && ny < mapHeight && !visited.has(key)) {
              const idx = ny * mapWidth + nx;
              if (mapData[idx] === 2) {
                visited.add(key);
                queue.push([nx, ny]);
              }
            }
          }
        }
      }
    }
    for (let y = 0; y < mapHeight; y++) {
      for (let x = 0; x < mapWidth; x++) {
        const idx = y * mapWidth + x;
        const isVisited = visitedData[idx] === 1;
        if (!isVisited) continue;
        const tile = mapData[idx];
        let color;
        if (tile === 1) color = '#555';
        else if (tile === 2) color = '#3a5';
        else if (tile === 5) color = '#62c';
        else if (tile === 3) color = '#dd0';
        else if (tile === 4) color = '#0dd';
        else color = '#4a6';
        if (roomTiles.has(`${x},${y}`)) {
          if (tile === 1) color = '#888';
          else if (tile === 2) color = '#6e8';
          else if (tile === 5) color = '#96f';
          else if (tile === 3) color = '#ff0';
          else if (tile === 4) color = '#0ff';
          else color = '#7b9';
        }
        ctx.fillStyle = color;
        ctx.fillRect(x * cellW, y * cellH, cellW, cellH);
      }
    }
    ctx.fillStyle = '#0f0';
    ctx.fillRect(playerX * cellW + 2, playerY * cellH + 2, cellW - 4, cellH - 4);
    ctx.fillStyle = '#0ff';
    ctx.font = '12px monospace';
    ctx.fillText(`F${wasm.depth_roguelike()}`, 8, 20);
    lastPlayerTile = currentPlayerTile;
  }

  return {
    updateStatusTab,
    updateEquipmentTab,
    selectEquipmentSlot,
    updateEquipmentDisplay,
    updateItemsDisplay,
    updateLogTab,
    updateMessageLog,
    setControlsSwapped,
    getDownRotation,
    flashHint,
    setupNavigation,
    updateTabContent,
    drawDungeonMap,
  };
}
