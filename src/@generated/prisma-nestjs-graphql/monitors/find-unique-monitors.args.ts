import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueMonitorsArgs {

    @Field(() => MonitorsWhereUniqueInput, {nullable:false})
    @Type(() => MonitorsWhereUniqueInput)
    where!: MonitorsWhereUniqueInput;
}
