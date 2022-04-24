import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsWhereUniqueInput } from './stats-where-unique.input';

@ArgsType()
export class FindUniqueStatsArgs {

    @Field(() => StatsWhereUniqueInput, {nullable:false})
    where!: StatsWhereUniqueInput;
}
