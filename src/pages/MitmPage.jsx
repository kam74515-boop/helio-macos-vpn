import { useState } from "react";
import { Toggle, Icon, CheckItem } from "../components/ui";

export function MitmPage() {
  const [enabled, setEnabled] = useState(false);
  return (
    <div className="page settings-page">
      <div className="headline-toggle"><h1>HTTPS 解密</h1><Toggle checked={enabled} onChange={setEnabled} /></div>
      <p className="muted">通过中间人攻击（MitM, man-in-the-middle）的方式解密 HTTPS 流量。</p>
      <div className="two-col">
        <section>
          <h2 className="section-title orange">CA 证书</h2>
          <div className="cert-row"><Icon name="workspace_premium" /><div><strong>配置中没有 CA 证书</strong><span>CA 证书不被系统信任</span></div></div>
          <div className="form-actions"><span>操作:</span><button>生成新证书</button><button disabled>将证书安装到系统</button><button>从 PKCS #12 文件导入证书</button><button disabled>为 iOS 模拟器导出证书</button></div>
          <h2 className="section-title green">选项</h2>
          <CheckItem label="进行 MitM 时跳过服务端证书验证" disabled />
          <CheckItem label="自动屏蔽 QUIC" disabled />
          <CheckItem label="MitM over HTTP/2" />
        </section>
        <section>
          <div className="host-box"><div>MitM 主机名 <Icon name="add_circle" /></div></div>
          <p className="muted">只有该列表中的域名才会被 Helio 进行解密，允许使用通配符。</p>
        </section>
      </div>
    </div>
  );
}
