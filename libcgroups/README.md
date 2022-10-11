# CGroups

cgroups (control groups) 只在 Linux 下支持，Mac OS X 不支持。Docker 是通过在 Mac 环境先由 [hyperkit](https://github.com/docker/hyperkit) 创建虚拟化环境，再在此虚拟化环境下使用虚拟化创建的容器。Podman 是通过 QEMU。

安装 Docker 即安装好了 hyperkit。

## References

- [Linux CGroups](https://man7.org/linux/man-pages/man7/cgroups.7.html)
