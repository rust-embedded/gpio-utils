# The version of rust we want to use
RUST_VERSION=1.37.0
DOCKER_IMAGE=ragnaroek/rust-raspberry:$RUST_VERSION


# Location of the rust code
PROJECT=`pwd`

# Registry
CARGO=$HOME/.cargo/registry

docker pull $DOCKER_IMAGE

case ".$1" in 
   ".build")
      docker run --name rpi-gpio-utils \
        --volume $PROJECT:/home/cross/project \
        --volume $CARGO:/home/cross/.cargo/registry \
        --volume $DEPS:/home/cross/deb-deps \
        $DOCKER_IMAGE \
        build --release
    ;;
   ".run")
      docker run -it --entrypoint /bin/bash \
        --volume $PROJECT:/home/cross/project \
        --volume $CARGO:/home/cross/.cargo/registry \
        --volume $DEPS:/home/cross/deb-deps \
	$DOCKER_IMAGE
    ;;
    *)
      echo "Pass one of the following arguments"
      echo ""
      echo "  build"
      echo "     Start a new container, build for raspbery-pi."
      echo "     Will leave app in target/arm-*/release/gpio"
      echo ""
      echo "  run"
      echo "    Start a new contained in interactive mode."
      echo "    Use this for debugging the build process."
    ;;
esac
