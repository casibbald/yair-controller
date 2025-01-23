# Usage default features:
# tilt up
#
# Usage with features:
# tilt up telemetry
config.define_string("features", args=True)
cfg = config.parse()
port = 11350

features = cfg.get('features', "")
print("compiling with features: {}".format(features))

local_resource('compile', 'just compile %s' % features)
local_resource('test', 'just test-unit')
docker_build('casibbald/yapp-controller', '.', dockerfile='Dockerfile')
# k8s_yaml('yaml/crd.yaml')
k8s_yaml('yaml/deployment.yaml')
k8s_resource('yapp-controller', port_forwards=8080)