# EXAMPLE
I think something in nvidia-smi should not available on Apple platform. 

This is an example for develop.

I deleted things I think should not be display. You can see description below.

**When someline finished, put char 'F' after that**

```
+-----------------------------------------------------------------------------------------+ F
| Apple-SMI 0.1.0                    macOS Version: 26.2           Metal Version: 4       | F
|-----------------------------------------+------------------------+----------------------| F
| GPU  Name                     Frequency | Bus-Id          Disp.A |                      | F
| Fan  Temp  Perf               Pwr:Usage |           Memory-Usage | GPU-Util  Compute M. | F
|                                         |                        |                      | F
|=========================================|========================+======================| F
|   0  Apple M4                   800 MHz | Built-in            On |                      | F
| 1000  34C  P3                      0.4W |     4096MiB / 16384MiB |      0%      Default |
|                                         |                        |                      |
+-----------------------------------------+------------------------+----------------------+
                                                                                            F
+-----------------------------------------------------------------------------------------+ F
| Processes:                                                                              |
|  GPU   PID   Type   Process name                                        GPU Memory      |
|=========================================================================================|
|  N/A  (system_profiler only: per-process GPU usage not available)                       |
+-----------------------------------------------------------------------------------------+
```

# Each item description:

## 顶部横幅（第一行）

常见长这样：

`NVIDIA-SMI 535.xx  Driver Version: 535.xx  CUDA Version: 12.x`

* **NVIDIA-SMI**：`nvidia-smi` 工具本身版本（属于 NVIDIA 工具链）。([NVIDIA Docs][1])

  * **Apple 平台**：不该出现（没有 NVSMI/NVML）。你可以替换成 `apple-smi <version>`。
* **Driver Version**：安装的 NVIDIA 显卡驱动版本。([NVIDIA Docs][1])

  * **Apple 平台**：不该出现（Apple Silicon 没有 NVIDIA 驱动；你可以显示 macOS 版本/构建号）。
* **CUDA Version**：该驱动支持的最高 CUDA 版本（不一定等于本机装的 toolkit）。([NVIDIA Docs][1])

  * **Apple 平台**：不该出现（没有 CUDA）。你可以显示 **Metal 版本/Metal Family**（例如你 JSON 里 `spdisplays_mtlgpufamilysupport`）。

---

## GPU 表（主表格）每一列

### `GPU`

* **含义**：GPU 的枚举索引（0,1,2…）。在 NVIDIA 里对应 NVML 的 device index。([NVIDIA Docs][1])
* **Apple 平台**：应出现（通常只有 0；多 GPU 机器可能多条）。

### `Name`

* **含义**：显卡产品名。([NVIDIA Docs][1])
* **Apple 平台**：应出现（例如 “Apple M4” / “Apple M2 Max”等，可来自 `system_profiler` 的 `_name`/`sppci_model`）。

### `Persistence-M`

* **含义**：是否开启“持久化模式”（让驱动在没有客户端时也保持加载，减少下一次启动 CUDA 程序的驱动加载延迟）。([NVIDIA Docs][1])
* **Apple 平台**：不该出现（这是 NVIDIA Linux 驱动/守护进程语义）。

### `Bus-Id`

* **含义**：PCI Bus ID（`domain:bus:device.function` 十六进制形式）。([NVIDIA Docs][1])
* **Apple 平台**：

  * **Apple Silicon（SoC 内建 GPU）**：严格来说“不该按 PCI BusId 出现”，但你可以放一个兼容字段：`Built-in` / `SoC` / `spdisplays_builtin`（你 JSON 里就有 `sppci_bus`）。
  * **Intel Mac + 独显/外接 eGPU**：可能有 PCI 概念（但这就不是 Apple Silicon 典型路径了）。

### `Disp.A`（Display Active）

* **含义**：显示是否在该 GPU 上被初始化（即使没插显示器也可能 Active）。([NVIDIA Docs][1])
* **Apple 平台**：**可选**

  * 你如果不想读显示器信息，可以直接 `N/A`；
  * 如果你愿意读一点点显示管线状态，可以用 system_profiler 的 display/online/main 等字段推断一个 “Active/Off”。

### `Volatile Uncorr. ECC`

* **含义**：ECC **易失计数（volatile）**维度里，**不可纠正（uncorrectable / double-bit）**错误的计数；“volatile”表示“自上次驱动加载以来”的计数窗口。([NVIDIA Docs][1])
* **Apple 平台**：不该出现（Apple Silicon GPU 没有你能以同样方式暴露的 ECC 计数；就算有内部纠错，也不是这个 NVML/ECC 模型）。

### `Fan`

* **含义**：风扇转速百分比（目标/意图转速，可能 >100%，且很多卡不支持上报）。([NVIDIA Docs][1])
* **Apple 平台**：多数情况下不该出现

  * Apple Silicon：风扇是整机散热，不是“GPU 独立风扇”；你最多显示整机风扇（也未必能无特权拿到）。
  * 从`powermetrics`拿数据

### `Temp`

* **含义**：温度传感器读数（常见显示 GPU core 当前温度，单位 °C；并非所有产品都支持）。([NVIDIA Docs][1])
* **Apple 平台**：**可以作为可选项**

  * 用 `system_profiler` 拿不到实时温度；
  * 能否拿到取决于你后续走不走 powermetrics/IOKit/传感器接口（很多都要权限）。
  * 现阶段（只做 system_profiler）建议 `N/A`。

### `Perf`（Performance State / P-state）

* **含义**：性能状态 P0..P12（P0 最高性能）。([NVIDIA Docs][1])
* **Apple 平台**：P-State，能拿到。

### `Pwr:Usage/Cap`

* **含义**：功耗读数（Usage）与功耗上限（Cap/Limit）。`nvidia-smi` 文档里把功耗作为 “Power Draw / Power Limit” 一类指标定义。([NVIDIA Docs][1])
* **Apple 平台**：**可以作为可选项**

  * system_profiler 给不了实时功耗；
  * powermetrics 这类能给，但通常需要高权限。

### `Memory-Usage`

* **含义**：帧缓冲（FB）显存使用：Used / Total（以及有时的 Reserved/Free）。([NVIDIA Docs][1])
* **Apple 平台**：**不该以“显存”语义直接出现**（Apple Silicon 是 **Unified Memory**）

  * 获取显存占用，还叫Memory-Usage

### `GPU-Util`

* **含义**：过去一个采样周期内 GPU 上有 kernel 执行的时间百分比（利用率）。([NVIDIA Docs][1])
* **Apple 平台**：**可选**

  * system_profiler 拿不到；
  * 未来如果你走性能计数器/系统指标才可能有。争取拿到。

### `Compute M.`（Compute Mode）

* **含义**：是否允许多个 compute context：Default / Exclusive Process / Prohibited。([NVIDIA Docs][1])
* **Apple 平台**：不该出现（这是 CUDA/NVML 的 compute context 管理语义；Metal 不这么叫）。

### `MIG M.`（MIG Mode）

* **含义**：Multi-Instance GPU 是否启用（Enabled/Disabled/NA）。([NVIDIA Docs][1])
* **Apple 平台**：不该出现（Apple 没有 MIG 这种分区机制对标）。

## MultiCard
Looks like this, so **ui render can draw card info from `S` to `E`**:
```
+-----------------------------------------------------------------------------+
| NVIDIA-SMI 470.161.03   Driver Version: 470.161.03   CUDA Version: 11.4     |
|-------------------------------+----------------------+----------------------+
| GPU  Name        Persistence-M| Bus-Id        Disp.A | Volatile Uncorr. ECC |
| Fan  Temp  Perf  Pwr:Usage/Cap|         Memory-Usage | GPU-Util  Compute M. |
|                               |                      |               MIG M. |
|===============================+======================+======================|
|   0  Tesla V100-SXM2...  On   | 00000000:00:09.0 Off |                    0 | S
| N/A   38C    P0    61W / 300W |    569MiB /  4309MiB |      2%      Default |
|                               |                      |                  N/A |
+-------------------------------+----------------------+----------------------+ E
|   1  Tesla V100-SXM2...  On   | 00000000:00:0A.0 Off |                    0 | S
| N/A   36C    P0    61W / 300W |    381MiB /  4309MiB |      0%      Default |
|                               |                      |                  N/A |
+-------------------------------+----------------------+----------------------+ E
```