#include <string>

namespace engine {

class Renderer {
public:
    Renderer(int width, int height);
    void draw();
private:
    int width_;
    int height_;
};

void initialize(const std::string& config) {
    // init
}

} // namespace engine
