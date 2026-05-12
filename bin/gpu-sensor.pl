#!/usr/bin/perl -w

# act as a KSysGuard sensor
# provides NVIDIA GPU info via `nvidia-settings`

# Usage (e.g. add gpu temperature sensor)
# 1. save this file, make sure it has a exec permission
# 2. in KSysGuard's menu, open `File` -> `Monitor Remote Machine`
# 3.1 in new dialog, type `Host` whatever you want
# 3.2 set `Connection Type` to `Custom command`
# 3.3 set `Command` like `path/to/this-sensor.pl`
# 4. click `OK`, now you can find new sensor named `gpu_temp`
#    which is provides GPU temperature

# See Also
# https://techbase.kde.org/Development/Tutorials/Sensors


$|=1;

print "ksysguardd 1.2.0\n";
print "ksysguardd> ";

while(<>){
    if(/monitors/){
        print "gpu_temp\tinteger\n";
        print "gpu_graphics\tinteger\n";
        print "gpu_memory\tinteger\n";
        print "gpu_video_engine\tinteger\n";
    }
    if(/gpu_temp/){
        if(/\?/){
            print "GPU Temp\t0\t0\n";
        }else{
            print `nvidia-settings -tq gpucoretemp | head -n1`;
        }
    }
    if(/gpu_graphics/){
        if(/\?/){
            print "GPU\t0\t0\n";
        }else{
            print `nvidia-settings -tq [gpu:0]/GPUUtilization | awk -F"," '{print(substr(\$1,index(\$1,"=")+1))}'`;
        }
    }
    if(/gpu_memory/){
        if(/\?/){
            print "GPU Memory\t0\t0\n";
        }else{
            print `nvidia-settings -tq [gpu:0]/GPUUtilization | awk -F"," '{print(substr(\$2,index(\$2,"=")+1))}'`;
        }
    }
    if(/gpu_video_engine/){
        if(/\?/){
            print "Video Engine\t0\t0\n";
        }else{
            print `nvidia-settings -tq [gpu:0]/GPUUtilization | awk -F"," '{print(substr(\$3,index(\$3,"=")+1))}'`;
        }
    }
    print "ksysguardd> ";
}
