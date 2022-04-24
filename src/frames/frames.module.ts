import { Module } from '@nestjs/common';
import { FramesService } from './frames.service';
import { FramesResolver } from './frames.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,FramesResolver, FramesService]
})
export class FramesModule {}
