import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersCreateManyInput } from './servers-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyServersArgs {

    @Field(() => [ServersCreateManyInput], {nullable:false})
    @Type(() => ServersCreateManyInput)
    data!: Array<ServersCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
