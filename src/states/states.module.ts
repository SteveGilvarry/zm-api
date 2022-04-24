import { Module } from '@nestjs/common';
import { StatesService } from './states.service';
import { StatesResolver } from './states.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, StatesResolver, StatesService]
})
export class StatesModule {}
