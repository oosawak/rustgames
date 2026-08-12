export function createDungeonLoadUi({ wasm }) {
  let selectedEquipmentSlot = 'weapon';
  let lastPlayerTile = -1;

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
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">レベル</div>
      <div style="font-size: 20px; color: #0ff; font-weight: bold;">${level}</div>
    </div>`;
    html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">🗻</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">深度</div>
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
      <div style="font-size: 12px; color: rgba(255,200,0,0.7); margin-bottom: 4px;">攻撃</div>
      <div style="font-size: 20px; color: #fd0; font-weight: bold;">+${atkBonus}</div>
    </div>`;
    html += `<div style="border: 1px solid rgba(150,150,150,0.3); border-radius: 4px; padding: 12px; text-align: center;">
      <div style="font-size: 24px; margin-bottom: 4px;">🛡️</div>
      <div style="font-size: 12px; color: rgba(150,150,150,0.7); margin-bottom: 4px;">防御</div>
      <div style="font-size: 20px; color: #aaa; font-weight: bold;">+${defBonus}</div>
    </div>`;
    html += '</div>';
    panel.innerHTML = html;
  }

  function updateEquipmentTab(panel) {
    if (!panel) return;
    const weaponNames = ['Wooden Sword', 'Iron Sword', 'Axe', 'Cursed Blade', 'Dragon Slayer'];
    const armorNames = ['Leather Armor', 'Chain Mail', 'Steel Plate', 'Dragon Scale', 'Cursed Mail'];
    const accessoryNames = ['Gold Ring', 'Vampire Ring', 'Lucky Ring', 'Healing Necklace', 'Mana Earrings'];
    const weaponBonuses = [3, 5, 7, 9, 12];
    const armorBonuses = [2, 4, 6, 8, 10];
    const wepIdx = wasm.player_equipped_weapon_roguelike();
    const armIdx = wasm.player_equipped_armor_roguelike();
    const accIdx = wasm.player_equipped_accessory_roguelike();
    let html = '<div style="padding: 16px;">';
    html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-bottom: 16px;">';
    html += `<div style="border: 2px solid ${selectedEquipmentSlot === 'weapon' ? '#0ff' : 'rgba(0,200,255,0.3)'}; border-radius: 4px; padding: 12px; text-align: center; cursor: pointer;" onclick="selectEquipmentSlot('weapon', event)">
      <div style="font-size: 24px; margin-bottom: 4px;">⚔️</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">武器</div>
      <div style="font-size: 11px; color: #0ff;">${wepIdx >= 0 ? weaponNames[wepIdx] : 'なし'}</div>
    </div>`;
    html += `<div style="border: 2px solid ${selectedEquipmentSlot === 'armor' ? '#0ff' : 'rgba(0,200,255,0.3)'}; border-radius: 4px; padding: 12px; text-align: center; cursor: pointer;" onclick="selectEquipmentSlot('armor', event)">
      <div style="font-size: 24px; margin-bottom: 4px;">🛡️</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">防具</div>
      <div style="font-size: 11px; color: #0ff;">${armIdx >= 0 ? armorNames[armIdx] : 'なし'}</div>
    </div>`;
    html += `<div style="border: 2px solid ${selectedEquipmentSlot === 'accessory' ? '#0ff' : 'rgba(0,200,255,0.3)'}; border-radius: 4px; padding: 12px; text-align: center; cursor: pointer;" onclick="selectEquipmentSlot('accessory', event)">
      <div style="font-size: 24px; margin-bottom: 4px;">💍</div>
      <div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 4px;">アクセサリ</div>
      <div style="font-size: 11px; color: #0ff;">${accIdx >= 0 ? accessoryNames[accIdx] : 'なし'}</div>
    </div>`;
    html += '</div>';
    if (selectedEquipmentSlot === 'weapon') {
      const weapons = wasm.weapon_inventory_roguelike();
      html += '<div style="border-top: 1px solid rgba(0,200,255,0.2); padding-top: 12px;"><div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 8px;">使用可能な武器</div>';
      html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;">';
      for (let i = 0; i < weapons.length; i++) {
        html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px;" onclick="wasm.equip_weapon_roguelike(${i}); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">
          <div>${weaponNames[weapons[i]]}</div>
          <div style="color: rgba(0,200,255,0.5); font-size: 9px;">+${weaponBonuses[weapons[i]]}</div>
        </div>`;
      }
      html += `<div style="border: 1px solid rgba(200,0,0,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px; background: rgba(200,0,0,0.1);" onclick="wasm.unequip_weapon_roguelike(); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">解除</div>`;
      html += '</div></div>';
    } else if (selectedEquipmentSlot === 'armor') {
      const armors = wasm.armor_inventory_roguelike();
      html += '<div style="border-top: 1px solid rgba(0,200,255,0.2); padding-top: 12px;"><div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 8px;">使用可能な防具</div>';
      html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;">';
      for (let i = 0; i < armors.length; i++) {
        html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px;" onclick="wasm.equip_armor_roguelike(${i}); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">
          <div>${armorNames[armors[i]]}</div>
          <div style="color: rgba(0,200,255,0.5); font-size: 9px;">+${armorBonuses[armors[i]]}</div>
        </div>`;
      }
      html += `<div style="border: 1px solid rgba(200,0,0,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px; background: rgba(200,0,0,0.1);" onclick="wasm.unequip_armor_roguelike(); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">解除</div>`;
      html += '</div></div>';
    } else if (selectedEquipmentSlot === 'accessory') {
      const accessories = wasm.accessory_inventory_roguelike();
      html += '<div style="border-top: 1px solid rgba(0,200,255,0.2); padding-top: 12px;"><div style="font-size: 12px; color: rgba(0,200,255,0.7); margin-bottom: 8px;">使用可能なアクセサリ</div>';
      html += '<div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;">';
      for (let i = 0; i < accessories.length; i++) {
        html += `<div style="border: 1px solid rgba(0,200,255,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px;" onclick="wasm.equip_accessory_roguelike(${i}); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">
          <div>${accessoryNames[accessories[i]]}</div>
        </div>`;
      }
      html += `<div style="border: 1px solid rgba(200,0,0,0.3); border-radius: 4px; padding: 8px; text-align: center; cursor: pointer; font-size: 10px; background: rgba(200,0,0,0.1);" onclick="wasm.unequip_accessory_roguelike(); updateEquipmentTab(document.getElementById('tab-equipment')); updateEquipmentDisplay();">解除</div>`;
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
    const itemNames = ['回復ポーション', 'マナポーション', '毒ポーション', 'エナジードリンク', '宝石', '鍵', 'スクロール', '金貨'];
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
      msgLogEl.innerHTML = messages
        .slice(-4)
        .map(msg => '<div style="margin: 2px 0; word-break: break-word;">' + msg + '</div>')
        .join('');
      msgLogEl.scrollTop = msgLogEl.scrollHeight;
    } else {
      msgLogEl.style.display = 'none';
    }
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
    navBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        const tabName = btn.getAttribute('data-tab');
        navBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
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
      case 'map':
        drawDungeonMap();
        break;
    }
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
        else if (tile === 3) color = '#dd0';
        else if (tile === 4) color = '#0dd';
        else color = '#4a6';
        if (roomTiles.has(`${x},${y}`)) {
          if (tile === 1) color = '#888';
          else if (tile === 2) color = '#6e8';
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
    updateMessageLog,
    flashHint,
    setupNavigation,
    updateTabContent,
    drawDungeonMap,
  };
}
