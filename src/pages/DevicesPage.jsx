import { useState } from "react";
import { SplitPage, Toggle, Icon } from "../components/ui";

export function DeviceDetail({ gateway }) {
  return (
    <aside className="detail-panel device-detail">
      <div className="detail-hero">
        <span className={`status-orb ${gateway ? "active" : ""}`} />
        <div>
          <h2>{gateway ? "网关待发现" : "设备列表为空"}</h2>
          <p>{gateway ? "网关模式开启后，新设备会显示在这里并可单独分配策略。" : "开启网关模式后，局域网设备会出现在此面板。"}</p>
        </div>
      </div>
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
  return (
    <SplitPage title="设备" control="按 IP 排序">
      <div className="empty-pane">
        <span>无设备</span>
        <div className="metered-row bottom">
          <div>
            <strong>网关模式</strong>
            <p>可以使用 Helio 作为局域网的 DHCP 服务器，接管其他设备的网络。</p>
          </div>
          <Toggle checked={gateway} onChange={setGateway} />
          <Icon name="settings" />
        </div>
      </div>
      <DeviceDetail gateway={gateway} />
    </SplitPage>
  );
}
