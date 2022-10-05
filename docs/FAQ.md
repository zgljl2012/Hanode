# FAQ

## How to get user and group on MacOS

```bash

id

```

## MacOS 报错: We cannot safely call it or ignore it in the fork() child process. Crashing instead. Set a breakpoint on objc_initializeAfterForkError to debug

```bash

export OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES

```
