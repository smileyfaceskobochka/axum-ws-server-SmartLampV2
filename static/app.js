class LampController {
    constructor() {
        this.ws = new WebSocket('ws://192.168.3.4:8000/ws/client');
        this.initElements();
        this.initEvents();
        this.initWebSocketHandlers();
    }

    initElements() {
        this.statusDot = document.getElementById('statusDot');
        this.connectionStatus = document.getElementById('connectionStatus');
        this.deviceStatus = document.getElementById('deviceStatus');
        this.toggleBtn = document.getElementById('toggleBtn');
        this.brightnessSlider = document.getElementById('brightness');
        this.colorPicker = document.getElementById('colorPicker');
    }

    initEvents() {
        this.toggleBtn.addEventListener('click', () => this.togglePower());
        this.brightnessSlider.addEventListener('input', (e) => this.updateBrightness(e));
        this.colorPicker.addEventListener('input', (e) => this.updateColor(e));
    }

    initWebSocketHandlers() {
        this.ws.onopen = () => this.updateConnectionStatus(true);
        this.ws.onerror = () => this.updateConnectionStatus(false);
        this.ws.onclose = () => this.updateConnectionStatus(false);
        this.ws.onmessage = (e) => this.handleMessage(e);
    }

    togglePower() {
        const power = !this.toggleBtn.classList.contains('active');
        this.sendCommand({ type: 'set_power', device_id: 'ESP32_LAMP_001', power });
    }

    updateBrightness(e) {
        const brightness = parseInt(e.target.value);
        this.sendCommand({ type: 'set_brightness', device_id: 'ESP32_LAMP_001', brightness });
    }

    updateColor(e) {
        const color = this.hexToRgb(e.target.value);
        this.sendCommand({ type: 'set_color', device_id: 'ESP32_LAMP_001', color: [color.r, color.g, color.b] });
    }

    sendCommand(command) {
        if (this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(command));
        }
    }

    handleMessage(e) {
        const data = JSON.parse(e.data);
        if (data.type === 'status_update') {
            this.updateUI(data);
            this.deviceStatus.textContent = 'Device connected';
        } else if (data.type === 'error') {
            this.connectionStatus.textContent = `Error: ${data.message}`;
        }
    }

    updateUI(status) {
        this.toggleBtn.classList.toggle('active', status.power);
        this.brightnessSlider.value = status.brightness;
        this.colorPicker.value = this.rgbToHex(status.color);
    }

    updateConnectionStatus(connected) {
        this.statusDot.className = `status-dot ${connected ? 'connected' : ''}`;
        this.connectionStatus.textContent = connected ? 'Connected to server' : 'Disconnected';
    }

    hexToRgb(hex) {
        const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return {
            r: parseInt(result[1], 16),
            g: parseInt(result[2], 16),
            b: parseInt(result[3], 16)
        };
    }

    rgbToHex(color) {
        return `#${((1 << 24) + (color[0] << 16) + (color[1] << 8) + color[2]).toString(16).slice(1)}`;
    }
}

document.addEventListener('DOMContentLoaded', () => new LampController());