all: process generate

generate: generate.cpp
	g++ -o generate generate.cpp

process: process.cpp
	g++ -o process process.cpp

simple:
	g++ -o simple listing_0067_simple_haversine.cpp

clean:
	rm -f process generate
