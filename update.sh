#!/bin/bash

sudo scp -r ./src/main.rs ubuntu@ec2-34-237-0-241.compute-1.amazonaws.com:/opt/webapp/src/
sudo scp -r ./src/meta_requests.rs ubuntu@ec2-34-237-0-241.compute-1.amazonaws.com:/opt/webapp/src/
sudo scp -r ./src/structs.rs ubuntu@ec2-34-237-0-241.compute-1.amazonaws.com:/opt/webapp/src/
sudo scp -r ./src/tools.rs ubuntu@ec2-34-237-0-241.compute-1.amazonaws.com:/opt/webapp/src/