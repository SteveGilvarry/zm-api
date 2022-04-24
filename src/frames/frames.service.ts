import { Injectable } from '@nestjs/common';
import { FramesCreateInput} from '../@generated/prisma-nestjs-graphql/frames/frames-create.input';
import { FramesUpdateInput} from '../@generated/prisma-nestjs-graphql/frames/frames-update.input';
import { FramesWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/frames/frames-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class FramesService {
  constructor(private readonly prisma: PrismaService) {}

  create(createFrameInput: FramesCreateInput) {
    return this.prisma.frames.create({
      data: createFrameInput,
    });
  }

  findAll() {
    return this.prisma.frames.findMany();
  }

  findOne(framesWhereUniqueInput: FramesWhereUniqueInput) {
    return this.prisma.frames.findUnique({
      where: framesWhereUniqueInput,
    });
  }

  update(
    framesWhereUniqueInput: FramesWhereUniqueInput,
    updateFrameInput: FramesUpdateInput
  ) {
    return this.prisma.frames.update({
      where: framesWhereUniqueInput,
      data: updateFrameInput,
    });
  }

  remove(framesWhereUniqueInput: FramesWhereUniqueInput) {
    return this.prisma.frames.delete({
      where: framesWhereUniqueInput,
    });
  }
}
