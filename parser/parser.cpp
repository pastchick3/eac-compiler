#include "CBaseListener.h"
#include "CLexer.h"
#include "CParser.h"
#include "antlr4-runtime.h"

class EventListener : public CBaseListener {
   public:
    // void enterValue(ArrayInitParser::ValueContext *ctx) override {
    //     std::cout << "enter: " << ctx->getText() << std::endl;
    // }
};

extern "C" int parse() {
    // // std::ifstream stream;
    // // stream.open(argv[1]);
    // // antlr4::ANTLRInputStream input(stream);
    // antlr4::ANTLRInputStream input("{ 1 }");

    // ArrayInitLexer lexer(&input);
    // antlr4::CommonTokenStream tokens(&lexer);
    // ArrayInitParser parser(&tokens);
    // antlr4::tree::ParseTree *tree = parser.init();

    // // EchoListener listener;
    // // antlr4::tree::ParseTreeWalker::DEFAULT.walk(&listener, tree);
    // std::cout << tree->toStringTree(&parser) << std::endl;

    return 2;
}
