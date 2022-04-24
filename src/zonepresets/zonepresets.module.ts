import { Module } from '@nestjs/common';
import { ZonepresetsService } from './zonepresets.service';
import { ZonepresetsResolver } from './zonepresets.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,ZonepresetsResolver, ZonepresetsService]
})
export class ZonepresetsModule {}
