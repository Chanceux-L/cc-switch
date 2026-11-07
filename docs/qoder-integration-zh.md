# Qoder 接入 CC Switch

## 范围与现状

`qoder` 是独立应用 ID，目前仅接入 **MCP 管理**。Qoder Desktop 的模型接入走账号体系（云端登录），本地没有可切换的供应商配置文件，因此与 Claude Code、Codex 等不同，CC Switch 不提供 Qoder 的供应商切换、会话与用量功能。

| 功能    | 实现                                 | 数据来源 / 写入目标          |
| ------- | ------------------------------------ | ---------------------------- |
| 供应商  | 不支持（账号登录，无本地配置可切换） | —                            |
| MCP     | 导入、编辑、独立启用/禁用、删除、重新同步 | `~/.qoder/settings.json` 的 `mcpServers` |
| Skills  | 不接入（Qoder 经插件市场管理）       | —                            |
| 提示词  | 不接入                               | —                            |
| 会话    | 不接入                               | —                            |

## MCP 读写语义

Qoder 的 MCP 与其他配置（如 `enabledPlugins`）共存于 `~/.qoder/settings.json` 一个文件。CC Switch 只增删改 `mcpServers` 键，其余根级键原样保留；写入为原子替换，数据库提交失败时回滚文件。

与 MiniMax Code、Oh My Pi 等应用不同，Qoder 原生格式**没有 `enabled` 字段**：条目存在即视为启用，禁用等于移除条目。因此在 CC Switch 里取消勾选某个应用会直接从 `settings.json` 删除对应条目，重新勾选则写回。原生文件里手工添加的条目（含 `qoder_url` 等扩展字段）在合并时保留非传输字段（如 `timeout`），只更新传输字段（`type`/`command`/`args`/`env`/`url`/`headers`）。

支持 stdio 与 HTTP 两种传输；`type` 缺省时按是否含 `url`/`command` 推断。「从应用导入」会把 `settings.json` 里的原生条目纳入统一管理。

## 环境变量

`QODER_CONFIG_DIR` 可覆盖配置目录（默认 `~/.qoder`），仅用于测试与自定义安装。

## 数据持久化和兼容性

数据库版本从 24 升至 25，为 `mcp_servers` 增加 `enabled_qoder BOOLEAN NOT NULL DEFAULT 0`。新建数据库与升级迁移均覆盖；其他应用的开关和数据不变。`skills` 表不增加列（Qoder 未接入 Skills 同步）。
