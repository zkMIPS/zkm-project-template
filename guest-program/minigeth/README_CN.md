<a href='./README.md' >English</a> <a>中文</a> 

cannon-mips是一个MIPS虚拟机(从cannon项目移植)，用于生成MIPS程序的执行记录。

## 目录布局

```
minigeth -- 能够计算区块转换的独立“geth”
mipsevm -- 生成MIPS程序执行记录的MIPS的VM
```

## 先决条件

参考 [mips_circuit的先决条件](https://github.com/zkMIPS/mips_circuit#prerequisite)

## 构建

#使用下面make build构建

```
make build
```

## 生成执行记录

除非另有说明，否则以下命令应该在项目根目录下运行:

```
# 示例:在个人电脑计算从13284469 -> 13284470的transition
$ export BASEDIR=/tmp/cannon # 使用export命令配置的环境变量只在当前的terminal有效，一旦当前terminal关闭则失效

$ export NODE=<mainnet-node> # 默认值: https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161

$ mkdir -p /tmp/cannon # 新建cannon文件夹
$ minigeth/go-ethereum 13284469 # 获取13284469的transition

$ export POSTGRES_CONFIG="sslmode=<sslmode> user=<user> password=<password> host=<ip> port=<port> dbname=<db>"
   # 默认值: sslmode=disable user=postgres password=postgres host=localhost port=5432 dbname=postgres，记得将对应的password修改为您对应的postgres密码

# 生成1个MIPS跟踪
$ cd mipsevm
$ ./mipsevm -b 13284469 -s 1
```

**NOTE: 当您运行“minigeth/go-ethereum”时，您可能无法访问默认的NODE URL并触发 ```nil指针 ```解引用故障，您可以从https://www.alchemy.com/ 注册一个免费的NODE来替换它**

## mipsevm命令参数

命令: 

```
mipsevm [-h] [-b blocknum] [-e elf-path] [-s stepnum] [-r rate] [-d]
```

参数:

```
  -h             help info 帮助信息
  -b <blocknum>  blocknum for minigeth minigeth的区块数
  -e <elf-path>  MIPS程序精灵路径(当指定blocknum时，默认为最小化)
  -s <stepnum>   要运行的程序步骤数(默认为4294967295)
  -r <rate>      随机生成跟踪率(1/100000)(默认100000)
  -d             为指令序列启用调试输出
```

示例:

- 生成13284469块的前1000条指令的记录

```
./mipsevm -b 13284469 -s 1000   //[-r 100000]和[-e minigeth]可以作为默认值
```

- 为13284469块的前1000条指令生成1%率的记录

```
./mipsevm -b 13284469 -s 1000 -r 1000
```

## 许可证

大部分代码是MIT许可的，minigeth的是LGPL3。

注意:此代码未经审计。它绝不应该被用来确保任何生产环境，直到更多测试和审计已经完成。我没有在任何地方部署它，建议不要部署它，而且不保证任何形式的安全。
