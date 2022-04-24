import { Module } from '@nestjs/common';
import { MontagelayoutsService } from './montagelayouts.service';
import { MontagelayoutsResolver } from './montagelayouts.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,MontagelayoutsResolver, MontagelayoutsService]
})
export class MontagelayoutsModule {}
