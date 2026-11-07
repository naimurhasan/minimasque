import sys

from PIL import Image, ImageDraw, ImageFont

BAR = 40
BAR_COLOR = (32, 32, 32)
TAB_COLOR = (30, 30, 46)
TEXT_COLOR = (230, 230, 230)
FONT = "/System/Library/Fonts/HelveticaNeue.ttc"


def caption_buttons(draw, width):
    y = BAR // 2
    x = width - 46 * 3
    draw.line((x + 18, y, x + 28, y), fill=TEXT_COLOR)
    x += 46
    draw.rectangle((x + 18, y - 5, x + 28, y + 5), outline=TEXT_COLOR)
    x += 46
    draw.line((x + 18, y - 5, x + 28, y + 5), fill=TEXT_COLOR)
    draw.line((x + 18, y + 5, x + 28, y - 5), fill=TEXT_COLOR)


def tab(draw, title):
    font = ImageFont.truetype(FONT, 13)
    draw.rounded_rectangle((8, 6, 248, BAR + 8), radius=8, fill=TAB_COLOR)
    draw.text((22, 13), ">_", font=font, fill=TEXT_COLOR)
    draw.text((46, 13), title, font=font, fill=TEXT_COLOR)
    draw.text((226, 12), "×", font=font, fill=TEXT_COLOR)
    draw.text((264, 10), "+", font=ImageFont.truetype(FONT, 16), fill=TEXT_COLOR)


def frame(screen, title):
    width, height = screen.size
    framed = Image.new("RGB", (width, height + BAR), BAR_COLOR)
    framed.paste(screen, (0, BAR))
    draw = ImageDraw.Draw(framed)
    tab(draw, title)
    caption_buttons(draw, width)
    return framed


if __name__ == "__main__":
    source, target = sys.argv[1], sys.argv[2]
    frame(Image.open(source).convert("RGB"), "minimasque.exe").save(target)
