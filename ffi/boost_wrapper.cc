#include <boost/math/special_functions/beta.hpp>

extern "C" {
    double boost_ibeta_inv(double a, double b, double p) {
        return boost::math::ibeta_inv(a, b, p);
    }

    double boost_ibeta(double a, double b, double p) {
        return boost::math::ibeta(a, b, p);
    }
}