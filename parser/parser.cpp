#include <fstream>
#include <vector>

#include "CBaseListener.h"
#include "CLexer.h"
#include "CParser.h"
#include "antlr4-runtime.h"

extern "C" typedef struct {
    const char *tag;
    const char *text;
} Event;

extern "C" typedef struct {
    Event *data;
    size_t len;
} Events;

class EventListener : public CBaseListener {
   public:
    Event *getEventsPtr() { return this->events.data(); }

    size_t getEventsSize() { return this->events.size(); }

    void enterPrimaryExpression(
        CParser::PrimaryExpressionContext *ctx) override {
        this->emit_event("PrimaryExpression", ctx->getText().c_str());
    }

    void enterPostfixExpression(
        CParser::PostfixExpressionContext *ctx) override {
            std::cout << ctx->getText().c_str() << " - ";
            if (ctx->argumentExpressionList()) {
                std::cout << "yes" << std::endl;
            } else {
                std::cout << "no" << std::endl;
            }
        this->emit_event("EnterPostfixExpression", "");
    }

    void enterFunctionDefinition(
        CParser::FunctionDefinitionContext *ctx) override {
        this->emit_event("EnterFunction", "");
    }

    void exitFunctionDefinition(
        CParser::FunctionDefinitionContext *ctx) override {
        this->emit_event("ExitFunction", "");
    }

   private:
    std::vector<Event> events;

    void emit_event(const char *tag, const char *text) {
        Event event;
        event.tag = tag;
        event.text = text;
        this->events.push_back(event);
    }
};

extern "C" Events _parse(char *path) {
    std::ifstream file;
    file.open(path);
    antlr4::ANTLRInputStream input(file);
    CLexer lexer(&input);
    antlr4::CommonTokenStream tokens(&lexer);
    CParser parser(&tokens);
    antlr4::tree::ParseTree *tree = parser.compilationUnit();
    EventListener listener;
    antlr4::tree::ParseTreeWalker::DEFAULT.walk(&listener, tree);
    std::cout << tree->toStringTree(&parser) << std::endl;
    file.close();
    Events events;
    events.data = listener.getEventsPtr();
    events.len = listener.getEventsSize();
    return events;
}
