#!/bin/bash

echo "================源码格式化阶段================"
cargo fmt -v
echo "================源码格式化结束================"

echo "================测试阶段================"
cargo test
echo "================测试结束================"
