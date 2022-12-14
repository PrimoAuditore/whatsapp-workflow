#!/bin/bash

sudo scp -r ./src/main.rs ubuntu@ec2-3-236-70-96.compute-1.amazonaws.com:/home/ubuntu/app/src/
sudo scp -r ./src/meta_requests.rs ubuntu@ec2-3-236-70-96.compute-1.amazonaws.com:/home/ubuntu/app/src/
sudo scp -r ./src/structs.rs ubuntu@ec2-3-236-70-96.compute-1.amazonaws.com:/home/ubuntu/app/src/
sudo scp -r ./src/tools.rs ubuntu@ec2-3-236-70-96.compute-1.amazonaws.com:/home/ubuntu/app/src/
sudo scp -r ./Dockerfile ubuntu@ec2-3-236-70-96.compute-1.amazonaws.com:/home/ubuntu/app/
sudo scp -r ./Cargo.toml ubuntu@ec2-3-236-70-96.compute-1.amazonaws.com:/home/ubuntu/app/
sudo scp -r ./.dockerignore ubuntu@ec2-3-236-70-96.compute-1.amazonaws.com:/home/ubuntu/app/
