import { Module } from '@nestjs/common';
import { ModelsService } from './models.service';
import { ModelsResolver } from './models.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,ModelsResolver, ModelsService]
})
export class ModelsModule {}
