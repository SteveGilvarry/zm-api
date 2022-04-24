import { Module } from '@nestjs/common';
import { ControlpresetsService } from './controlpresets.service';
import { ControlpresetsResolver } from './controlpresets.resolver';
import { PrismaService } from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, ControlpresetsResolver, ControlpresetsService]
})
export class ControlpresetsModule {}
