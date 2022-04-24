import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesCreateManyInput } from './zones-create-many.input';

@ArgsType()
export class CreateManyZonesArgs {

    @Field(() => [ZonesCreateManyInput], {nullable:false})
    data!: Array<ZonesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
