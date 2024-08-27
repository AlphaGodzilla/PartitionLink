#!/bin/bash

cargo test

cargo fix --allow-dirty --allow-staged
