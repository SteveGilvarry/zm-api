import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsCreateManyInput } from './monitors-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyMonitorsArgs {

    @Field(() => [MonitorsCreateManyInput], {nullable:false})
    @Type(() => MonitorsCreateManyInput)
    data!: Array<MonitorsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
