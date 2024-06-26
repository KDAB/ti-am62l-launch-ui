#!/bin/bash
HOST=$1

ssh -t $HOST "sudo systemctl stop bb-code-server.service"
