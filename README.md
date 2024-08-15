# PartitionLink

## PL是什么？
PL是一个分布式的内存共享数据网格平台，支持多种语言内嵌运行和跨语言级别的共享内存。

##  和其他共享内存引擎的区别？
知名的共享内存方案有Redis，Gemfire，Geode，Hazelcast等。其中Redis是C/S架构[^1]，其他则是进程内嵌共享堆内存和C/S架构双支持方案，Gemfire，Geode，Hazelcast在使用进程内嵌架构时只能运行在JVM虚拟机之上。对于非Java或JVM环境则只能采用C/S架构。SME的目标是支持在非JVM堆内存中来共享内存。比如JVM进程和Rust共享内存或者Rust与Golang共享内存

## 目标
1. 跨语言级别的共享内存
2. 分布式计算MapReduce
3. 。。。


## 脚注
[^1]: C/S架构既Client和Server
