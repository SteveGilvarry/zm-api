import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsCreateInput } from './monitors-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneMonitorsArgs {

    @Field(() => MonitorsCreateInput, {nullable:false})
    @Type(() => MonitorsCreateInput)
    data!: MonitorsCreateInput;
}
