#include <algorithm>
#include <array>
#include <iomanip>
#include <iostream>
#include <random>

typedef double f64;
#include "listing_0065_haversine_formula.cpp"

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
    auto x0 = clip(cluster0.center_x + cluster0.radius(mt));
    auto y0 = clip(cluster0.center_y + cluster0.radius(mt));
    auto cluster1 = clusters[mt() % NUM_CLUSTERS];
    auto x1 = clip(cluster1.center_x + cluster1.radius(mt));
    auto y1 = clip(cluster1.center_y + cluster1.radius(mt));

    std::cout.setf(std::ios::fixed);
    std::cout << std::setprecision(16) << "{\"x0\":" << x0 << ",\"y0\":" << y0
              << ",\"x1\":" << x1 << ",\"y1\":" << y1 << "}";
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