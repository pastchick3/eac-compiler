#include <cstring>

#include "CBaseListener.h"
#include "CLexer.h"
#include "CParser.h"
#include "antlr4-runtime.h"

typedef char *(*RsGetStr)(size_t len);
typedef void (*RsEmitEvent)(char *tag, char *text);

class EventListener : public CBaseListener {
   public:
    EventListener(RsGetStr rsGetStr, RsEmitEvent rsEmitEvent)
        : rsGetStr(rsGetStr), rsEmitEvent(rsEmitEvent) {}

    void exitPrimaryExpression(
        CParser::PrimaryExpressionContext *ctx) override {
        this->emitEvent("ExitPrimaryExpression", ctx->getText().c_str());
    }

    void exitUnaryExpression(CParser::UnaryExpressionContext *ctx) override {
        if (auto op = ctx->unaryOperator()) {
            this->emitEvent("ExitUnaryExpression", op->getText().c_str());
        }
    }

    void exitPostfixExpression(
        CParser::PostfixExpressionContext *ctx) override {
        if (ctx->LeftParen() || ctx->argumentExpressionList()) {
            this->emitEvent("ExitPostfixExpression", "");
        }
    }

    void exitArgumentExpressionList(
        CParser::ArgumentExpressionListContext *ctx) override {
        this->emitEvent("ExitArgumentExpressionList", "");
    }

    void enterCompoundStatement(
        CParser::CompoundStatementContext *ctx) override {
        this->emitEvent("EnterCompoundStatement", "");
    }

    void exitCompoundStatement(
        CParser::CompoundStatementContext *ctx) override {
        this->emitEvent("ExitCompoundStatement", "");
    }

    void exitExpressionStatement(
        CParser::ExpressionStatementContext *ctx) override {
        this->emitEvent("ExitExpressionStatement", "");
    }

    void exitFunctionDefinition(
        CParser::FunctionDefinitionContext *ctx) override {
        std::string sig;
        if (ctx->declarationSpecifiers()
                ->declarationSpecifier(0)
                ->typeSpecifier()
                ->Void()) {
            sig.append("void");
        } else {
            sig.append("int");
        }
        sig.push_back(' ');
        auto name = ctx->declarator()
                        ->directDeclarator()
                        ->directDeclarator()
                        ->Identifier()
                        ->getText();
        sig.append(name);
        if (auto param_list =
                ctx->declarator()->directDeclarator()->parameterTypeList()) {
            auto parameter = param_list->parameterList();
            while (parameter) {
                auto param = parameter->parameterDeclaration()
                                 ->declarator()
                                 ->directDeclarator()
                                 ->getText();
                sig.push_back(' ');
                sig.append(param);
                parameter = parameter->parameterList();
            }
        }
        this->emitEvent("ExitFunctionDefinition", sig.c_str());
    }

   private:
    RsGetStr rsGetStr;
    RsEmitEvent rsEmitEvent;

    void emitEvent(const char *tag, const char *text) {
        auto rsTag{this->rsGetStr(std::strlen(tag))};
        std::strcpy(rsTag, tag);
        auto rsText{this->rsGetStr(std::strlen(text))};
        std::strcpy(rsText, text);
        this->rsEmitEvent(rsTag, rsText);
    }
};

extern "C" char *_parse(char *source, RsGetStr rsGetStr,
                        RsEmitEvent rsEmitEvent) {
    antlr4::ANTLRInputStream input{source};
    CLexer lexer{&input};
    antlr4::CommonTokenStream tokens{&lexer};
    CParser parser{&tokens};
    antlr4::tree::ParseTree *tree{parser.compilationUnit()};
    EventListener listener{rsGetStr, rsEmitEvent};
    antlr4::tree::ParseTreeWalker::DEFAULT.walk(&listener, tree);
    std::cout << tree->toStringTree(&parser)
              << std::endl;  //-------------------delete
    return source;
}
