#include <algorithm>
#include <array>
#include <iomanip>
#include <iostream>
#include <random>

// casey's calculation code, used for generating the expected sum
typedef double f64;

static f64 Square(f64 A) {
  f64 Result = (A * A);
  return Result;
}

static f64 RadiansFromDegrees(f64 Degrees) {
  f64 Result = 0.01745329251994329577f * Degrees;
  return Result;
}

// NOTE(casey): EarthRadius is generally expected to be 6372.8
static f64 ReferenceHaversine(f64 X0, f64 Y0, f64 X1, f64 Y1, f64 EarthRadius) {
  /* NOTE(casey): This is not meant to be a "good" way to calculate the
     Haversine distance. Instead, it attempts to follow, as closely as possible,
     the formula used in the real-world question on which these homework
     exercises are loosely based.
  */

  f64 lat1 = Y0;
  f64 lat2 = Y1;
  f64 lon1 = X0;
  f64 lon2 = X1;

  f64 dLat = RadiansFromDegrees(lat2 - lat1);
  f64 dLon = RadiansFromDegrees(lon2 - lon1);
  lat1 = RadiansFromDegrees(lat1);
  lat2 = RadiansFromDegrees(lat2);

  f64 a =
      Square(sin(dLat / 2.0)) + cos(lat1) * cos(lat2) * Square(sin(dLon / 2));
  f64 c = 2.0 * asin(sqrt(a));

  f64 Result = EarthRadius * c;

  return Result;
}

const u_int64_t NUM_CLUSTERS = 64;
const int MIN = 0;
const int MAX = 360;
std::uniform_real_distribution<double> CLUSTER_CENTER_DIST(MIN, MAX);
std::uniform_real_distribution<double> RADIUS_DIST(0.5, 50);

struct Cluster {
  double center_x;
  double center_y;
  std::uniform_real_distribution<double> radius;
};

Cluster rand_cluster(std::mt19937 &mt) {
  return Cluster{
      .center_x = CLUSTER_CENTER_DIST(mt),
      .center_y = CLUSTER_CENTER_DIST(mt),
      .radius = std::uniform_real_distribution<double>(-RADIUS_DIST(mt),
                                                       RADIUS_DIST(mt)),
  };
}

double clip(double n) {
  if (n < MIN) {
    n = MAX - n;
  }
  if (n > MAX) {
    n = MIN + n;
  }
  return n - 180;
}

int main(int argc, char **argv) {
  if (argc < 3) {
    std::cerr << "Usage: " << argv[0] << " <seed> <num_to_gen>" << std::endl;
    return 1;
  }
  auto seed = strtol(argv[1], nullptr, 10);
  auto num_to_gen = strtol(argv[2], nullptr, 10);

  std::mt19937 mt(seed);
  auto clusters = std::array<Cluster, NUM_CLUSTERS>{};
  std::generate(clusters.begin(), clusters.end(),
                [&mt]() { return rand_cluster(mt); });

  std::cout << "{\"pairs\":[" << std::endl;
  double sum = 0.;
  for (auto i = 0; i < num_to_gen; i++) {
    auto cluster0 = clusters[mt() % NUM_CLUSTERS];
    auto x0 = cluster0.center_x + cluster0.radius(mt);
    auto y0 = cluster0.center_y + cluster0.radius(mt);
    auto cluster1 = clusters[mt() % NUM_CLUSTERS];
    auto x1 = cluster1.center_x + cluster1.radius(mt);
    auto y1 = cluster1.center_y + cluster1.radius(mt);

    std::cout.setf(std::ios::fixed);
    std::cout << std::setprecision(16) << "{\"x0\":" << clip(x0)
              << ",\"y0\":" << clip(y0) << ",\"x1\":" << clip(x1)
              << ",\"y1\":" << clip(y1) << "}";

    sum += ReferenceHaversine(x0, y0, x1, y1, 6372.8);

    if (i != num_to_gen - 1) {
      std::cout << ",";
    }
    std::cout << std::endl;
  }
  std::cout << "]}" << std::endl;

  std::cerr.setf(std::ios::fixed);
  sum /= (double)num_to_gen;
  std::cerr << std::setprecision(16) << "Expected sum: " << sum << std::endl;
}