import { useState } from "react";
import { Toggle } from "../components/ui";
import { useToast } from "../components/Toast";

export function RewritePage() {
  const [enabled, setEnabled] = useState(true);
  const { addToast } = useToast();

  const rewrites = [
    ["URL 重定向", "重定向 HTTP 请求，也称为 Map Remote。", "编辑 URL 重定向规则...", () => addToast("URL 重定向功能开发中，需要配合 MITM 使用", "info")],
    ["Header 重写", "修改发送到服务器的 HTTP Header，以及修改返回响应的 Header。", "编辑 Header 重写规则...", () => addToast("Header 重写功能开发中", "info")],
    ["Mock", "模拟 API 服务器并返回静态响应。", "编辑 Mock 规则...", () => addToast("Mock 功能开发中", "info")],
    ["Body 重写", "重写 HTTP 请求或响应的 Body，用正则表达式替换原始内容。", "编辑 Body 重写规则...", () => addToast("Body 重写功能开发中", "info")],
  ];

  return (
    <div className="page airy">
      <div className="headline-toggle"><h1>重写 & 映射</h1><Toggle checked={enabled} onChange={setEnabled} /></div>
      <div className="rewrite-grid">
        {rewrites.map(([title, text, action, onClick]) => (
          <section className="card rewrite-card" key={title}>
            <h3>{title}</h3>
            <p>{text}</p>
            <div><span>暂不支持</span><button onClick={onClick}>{action}</button></div>
          </section>
        ))}
      </div>
      <p className="muted">以上计数包含了模块中的规则。</p>
    </div>
  );
}
