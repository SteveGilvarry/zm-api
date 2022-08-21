import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsWhereUniqueInput } from './stats-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueStatsArgs {

    @Field(() => StatsWhereUniqueInput, {nullable:false})
    @Type(() => StatsWhereUniqueInput)
    where!: StatsWhereUniqueInput;
}
