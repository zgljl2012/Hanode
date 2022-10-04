# 使用 Mininet 搭建测试环境

因为本项目最终场景为管理多台服务器，单机模式下无法进行完备测试，如果使用真实环境，不利于开发环境开发，虚拟机环境对机器内存要求高；基于容器组网足够轻量，但模拟各种网络情况难度较大、操作较繁琐。故综合考虑下，故使用 [Mininet](http://mininet.org/) 来为本项目搭建测试环境。

## Setup

```bash

docker pull iwaseyusuke/mininet:ubuntu-20.04

docker run -it --rm --privileged -e DISPLAY \
             -v /tmp/.X11-unix:/tmp/.X11-unix \
             -v /lib/modules:/lib/modules \
             iwaseyusuke/mininet:ubuntu-20.04

```

## 组网

[Help](https://xiaoer.blog.csdn.net/article/details/105230800)

```bash
# 进入 Minnet
mn

# 初始创建了 h1 和 h2，且创建了 s1 网关
# ((h1, s1) (h2, s1)
net

# 查看 h1 IP: 10.0.0.1
h1 ifconfig

# 查看 h2 IP: 10.0.0.2
h2 ifconfig

# link and ping
# 初始化 ping 不通
h1 ping h2
# dpctl
dpctl add-flow in_port=1,action=flood
dpctl add-flow in_port=2,action=flood
# 然后就能 ping 通了
pingall

# 执行外部 shell 命令
sh pwd
sh touch 1.py

# 执行 python 脚本
py <...>

py net.addLink(s1, s2)

```
