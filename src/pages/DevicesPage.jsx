import { useState } from "react";
import { SplitPage, Toggle, Icon } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri } from "../utils/tauri";
import { useToast } from "../components/Toast";

export function DeviceDetail({ gateway, devices = [] }) {
  return (
    <aside className="detail-panel device-detail">
      <div className="detail-hero">
        <span className={`status-orb ${gateway ? "active" : ""}`} />
        <div>
          <h2>{gateway ? "网关待发现" : "设备列表为空"}</h2>
          <p>{gateway ? "网关模式开启后，新设备会显示在这里并可单独分配策略。" : "开启网关模式后，局域网设备会出现在此面板。"}</p>
        </div>
      </div>
      {devices.length > 0 && (
        <div style={{ padding: "12px 0" }}>
          {devices.map((device) => (
            <div key={device.ip || device.mac} className="side-chip" style={{ justifyContent: "space-between" }}>
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {device.name || device.ip}
              </span>
              <span className="muted" style={{ fontSize: 12, flex: "0 0 auto" }}>
                {device.ip}{device.mac ? ` · ${device.mac}` : ""}
              </span>
            </div>
          ))}
        </div>
      )}
      <div className="device-steps">
        <div><Icon name="router" /><span>启用 DHCP 网关</span></div>
        <div><Icon name="alt_route" /><span>选择默认策略组</span></div>
        <div><Icon name="checklist" /><span>为设备绑定规则</span></div>
      </div>
    </aside>
  );
}

export function DevicesPage() {
  const [gateway, setGateway] = useState(false);
  const { addToast } = useToast();
  const { data: devices } = useTauriPoll("get_lan_devices", null, 5000);
  const realDevices = canUseTauri() && devices?.length ? devices : [];

  const handleGatewayChange = (next) => {
    if (next) {
      addToast("网关模式需要管理员权限，功能开发中", "info");
    } else {
      addToast("网关模式已关闭", "info");
    }
    setGateway(next);
  };

  return (
    <SplitPage title="设备" control="按 IP 排序">
      <div className="empty-pane">
        {realDevices.length === 0 ? <span>未发现局域网设备</span> : (
          <div style={{ padding: "8px 0" }}>
            {realDevices.map((device) => (
              <div key={device.ip || device.mac} className="side-chip" style={{ justifyContent: "space-between" }}>
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {device.name || device.ip}
                </span>
                <span className="muted" style={{ fontSize: 12, flex: "0 0 auto" }}>
                  {device.ip}{device.mac ? ` · ${device.mac}` : ""}
                </span>
              </div>
            ))}
          </div>
        )}
        <div className="metered-row bottom">
          <div>
            <strong>网关模式</strong>
            <p>可以使用 Helio 作为局域网的 DHCP 服务器，接管其他设备的网络。</p>
          </div>
          <Toggle checked={gateway} onChange={handleGatewayChange} />
          <Icon name="settings" />
        </div>
      </div>
      <DeviceDetail gateway={gateway} devices={realDevices} />
    </SplitPage>
  );
}
